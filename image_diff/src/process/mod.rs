mod average;
mod kmeans;
mod pixel;

use {
    crate::{
        utils::{ciede2000, converter, RawColor, HSV, SRBG},
        CalculationUnit, ColorSpace, DistanceAlgorithm,
    },
    async_std::task::{spawn_blocking, JoinHandle},
    average::AverageImpl,
    image::{imageops::FilterType, RgbImage},
    kmeans::KMeansImpl,
    palette::{Lab, Pixel},
    pixel::PixelImpl,
    std::{collections::VecDeque, fmt, path::PathBuf, sync::Arc},
};

pub type Mask = (u32, u32, u32, u32);
pub type Tasks<T> = VecDeque<JoinHandle<T>>;
pub type LibItem = (Vec<RawColor>, Arc<RgbImage>);

type Converter = Box<dyn Fn(&[u8; 3]) -> RawColor + Sync + Send>;
type Distance = Box<dyn Fn(&RawColor, &RawColor) -> f32 + Sync + Send>;

trait Process {
    fn size(&self) -> u32;

    fn filter(&self) -> FilterType;

    fn index_step(&self, img: RgbImage) -> LibItem;

    fn fill_step(
        &self,
        img: Arc<RgbImage>,
        mask: Mask,
        lib: Arc<Vec<LibItem>>,
    ) -> (Mask, Arc<RgbImage>);
}

pub struct ProcessWrapper(Arc<dyn Process + Sync + Send + 'static>);

impl ProcessWrapper {
    #[inline(always)]
    pub fn new(
        size: u32,
        k: usize,
        hamerly: bool,
        calc_unit: CalculationUnit,
        color_space: ColorSpace,
        dist_algo: DistanceAlgorithm,
        filter: FilterType,
    ) -> Self {
        let distance = Box::new(match dist_algo {
            DistanceAlgorithm::Euclidean => match color_space {
                ColorSpace::HSV => |a: &RawColor, b: &RawColor| {
                    let a = HSV::from_raw(a);
                    let b = HSV::from_raw(b);
                    (a.hue.to_positive_degrees() - b.hue.to_positive_degrees()).powi(2)
                        + ((a.saturation - b.saturation) * 360.).powi(2)
                        + ((a.value - b.value) * 360.).powi(2)
                },
                _ => |a: &RawColor, b: &RawColor| {
                    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
                },
            },
            DistanceAlgorithm::CIEDE2000 => match color_space {
                ColorSpace::RGB => ciede2000::<SRBG>,
                ColorSpace::HSV => ciede2000::<HSV>,
                ColorSpace::CIELAB => ciede2000::<Lab>,
            },
        });

        let converter = Box::new(match color_space {
            ColorSpace::RGB => converter::<SRBG>,
            ColorSpace::HSV => converter::<HSV>,
            ColorSpace::CIELAB => converter::<Lab>,
        });

        Self(match calc_unit {
            CalculationUnit::Average => {
                Arc::new(AverageImpl::new(size, filter, converter, distance))
            }
            CalculationUnit::Pixel => Arc::new(PixelImpl::new(size, filter, converter, distance)),
            CalculationUnit::KMeans => Arc::new(KMeansImpl::new(
                size,
                k,
                hamerly,
                filter,
                distance,
                color_space,
            )),
        })
    }

    #[inline(always)]
    fn inner(&self) -> Arc<dyn Process + Sync + Send + 'static> {
        self.0.clone()
    }

    #[inline(always)]
    pub fn index(&self, libraries: &[PathBuf]) -> Tasks<Option<LibItem>> {
        libraries
            .iter()
            .map(|lib| {
                let lib = lib.clone();
                let inner = self.inner();
                spawn_blocking(move || {
                    if let Ok(img) = image::open(lib) {
                        let img = img
                            .resize_to_fill(inner.size(), inner.size(), inner.filter())
                            .into_rgb8();
                        return Some(inner.index_step(img));
                    }
                    None
                })
            })
            .collect::<VecDeque<_>>()
    }

    #[inline(always)]
    pub fn fill(
        &self,
        img: Arc<RgbImage>,
        lib: Arc<Vec<LibItem>>,
        masks: &Vec<Mask>,
    ) -> Tasks<(Mask, Arc<RgbImage>)> {
        masks
            .iter()
            .map(|&mask| {
                let img = img.clone();
                let lib = lib.clone();
                let inner = self.inner();
                spawn_blocking(move || inner.fill_step(img, mask, lib))
            })
            .collect::<VecDeque<_>>()
    }

    #[inline(always)]
    pub fn mask(size: u32, img: &RgbImage) -> Vec<Mask> {
        let (width, height) = img.dimensions();
        let mut mask = Vec::with_capacity((((width / size) + 1) * ((height / size) + 1)) as usize);
        for y in (0..height).step_by(size as usize) {
            for x in (0..width).step_by(size as usize) {
                let w = size.min(width - x);
                let h = size.min(height - y);
                mask.push((x, y, w, h));
            }
        }
        mask
    }
}

impl fmt::Debug for ProcessWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("").field(&"Process").finish()
    }
}

#[cfg(test)]
mod tests {
    use {
        async_std::task::block_on,
        image::{self, imageops::FilterType, ImageBuffer, RgbImage},
        std::{fs::read_dir, path::PathBuf, sync::Arc},
    };

    #[test]
    fn process() {
        let size = 50;
        let k = 1;
        let hamerly = false;
        let proc = super::ProcessWrapper::new(
            size,
            k,
            hamerly,
            crate::CalculationUnit::Average,
            crate::ColorSpace::CIELAB,
            crate::DistanceAlgorithm::CIEDE2000,
            FilterType::Nearest,
        );
        let library = read_dir("../image_crawler/test")
            .unwrap()
            .filter_map(|res| match res.as_ref() {
                Ok(entry) => {
                    let path = entry.path();
                    let ext = path
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default();
                    if path.is_file() && ["png", "jpg", "jpeg"].contains(&ext) {
                        return Some(path);
                    }
                    None
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        block_on(async move {
            let mut lib = Vec::with_capacity(library.len());
            let tasks = proc.index(&library);
            for task in tasks {
                if let Some(i) = task.await {
                    lib.push(i);
                }
            }
            let lib = Arc::new(lib);

            let img = Arc::new(
                image::open(PathBuf::from("../static/images/testdata.jpg"))
                    .unwrap()
                    .into_rgb8(),
            );
            let masks = super::ProcessWrapper::mask(size, &img);
            let (width, height) = img.dimensions();

            let tasks = proc.fill(img, lib, &masks);
            let mut img_buf: RgbImage = ImageBuffer::new(width, height);
            for task in tasks {
                let ((x, y, w, h), replace) = task.await;
                for j in 0..h {
                    for i in 0..w {
                        let p = replace.get_pixel(i, j);
                        img_buf.put_pixel(i + x, j + y, *p);
                    }
                }
            }
            img_buf.save("test.png").unwrap();
        });
    }
}

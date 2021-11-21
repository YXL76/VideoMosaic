mod average;
mod kmeans;
mod pixel;

use {
    crate::{
        utils::{ciede2000, converter, FrameIter, RawColor, HSV, SRBG},
        CalculationUnit, ColorSpace, DistanceAlgorithm, ImageDump, Transcode,
    },
    async_std::task::{spawn_blocking, JoinHandle},
    average::AverageImpl,
    futures::stream::{futures_unordered, FuturesUnordered},
    image::{imageops::FilterType, ImageBuffer, RgbImage},
    kmeans::KMeansImpl,
    palette::{Lab, Pixel},
    pixel::PixelImpl,
    std::{fmt, path::PathBuf, sync::Arc},
};

pub type Mask = (u32, u32, u32, u32);
type Task<T> = JoinHandle<T>;
type Tasks<T> = FuturesUnordered<Task<T>>;
pub type TasksIter<T> = futures_unordered::IntoIter<Task<T>>;
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

pub struct ProcessWrapper {
    iter: Box<dyn FrameIter + Sync + Send + 'static>,
    inner: Arc<dyn Process + Sync + Send + 'static>,
    buf: RgbImage,
    lib: Option<Arc<Vec<LibItem>>>,
    masks: Option<Vec<Mask>>,
}

impl ProcessWrapper {
    #[inline(always)]
    pub fn new(
        ProcessConfig {
            size,
            k,
            hamerly,
            calc_unit,
            color_space,
            dist_algo,
            filter,
        }: ProcessConfig,
        input: String,
        output: String,
        video: bool,
    ) -> (Self, (i64, u32, u32)) {
        let size = size.into();
        let filter = filter.into();

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

        let inner: Arc<dyn Process + Sync + Send + 'static> = match calc_unit {
            CalculationUnit::Average => {
                Arc::new(AverageImpl::new(size, filter, converter, distance))
            }
            CalculationUnit::Pixel => Arc::new(PixelImpl::new(size, filter, converter, distance)),
            CalculationUnit::KMeans => Arc::new(KMeansImpl::new(
                size,
                k.into(),
                hamerly,
                filter,
                distance,
                color_space,
            )),
        };

        if video {
            let (iter, info) = Transcode::new(input, output);
            (
                Self {
                    iter: Box::new(iter),
                    inner,
                    buf: ImageBuffer::new(info.1, info.2),
                    lib: None,
                    masks: None,
                },
                info,
            )
        } else {
            let (iter, info) = ImageDump::new(input, output);
            (
                Self {
                    iter: Box::new(iter),
                    inner,
                    buf: ImageBuffer::new(info.1, info.2),
                    lib: None,
                    masks: None,
                },
                info,
            )
        }
    }

    #[inline(always)]
    fn inner(&self) -> Arc<dyn Process + Sync + Send + 'static> {
        self.inner.clone()
    }

    #[inline(always)]
    pub fn index(&self, libraries: Vec<PathBuf>) -> Tasks<Option<LibItem>> {
        libraries
            .into_iter()
            .map(|lib| {
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
            .collect::<FuturesUnordered<_>>()
    }

    #[inline(always)]
    pub fn pre_fill(&mut self, lib: Arc<Vec<LibItem>>) {
        self.lib = Some(lib);
        self.masks = Some(self.mask());
    }

    #[inline(always)]
    pub fn fill(&mut self) -> Option<Tasks<(Mask, Arc<RgbImage>)>> {
        let lib = self.lib.as_ref().unwrap();
        match self.iter.next() {
            Some(img) => Some(
                self.masks
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|&mask| {
                        let img = img.clone();
                        let lib = lib.clone();
                        let inner = self.inner();
                        spawn_blocking(move || inner.fill_step(img, mask, lib))
                    })
                    .collect::<FuturesUnordered<_>>(),
            ),
            None => {
                self.iter.flush();
                None
            }
        }
    }

    #[inline(always)]
    pub fn post_fill_step(&mut self, (x, y, w, h): Mask, replace: Arc<RgbImage>) {
        for j in 0..h {
            for i in 0..w {
                let p = replace.get_pixel(i, j);
                self.buf.put_pixel(i + x, j + y, *p);
            }
        }
    }

    #[inline(always)]
    pub fn post_fill(&mut self) {
        self.iter.post_next(&self.buf)
    }

    #[inline(always)]
    fn mask(&self) -> Vec<Mask> {
        let size = self.inner.size();
        let (width, height) = self.buf.dimensions();
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

#[derive(Copy, Clone, Debug)]
pub struct ProcessConfig {
    pub size: u16,
    pub k: u8,
    pub hamerly: bool,
    pub calc_unit: CalculationUnit,
    pub color_space: ColorSpace,
    pub dist_algo: DistanceAlgorithm,
    pub filter: Filter,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            size: 100,
            k: 1,
            hamerly: false,
            calc_unit: Default::default(),
            color_space: Default::default(),
            dist_algo: Default::default(),
            filter: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Filter {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

impl Default for Filter {
    fn default() -> Self {
        Self::Nearest
    }
}

impl From<Filter> for FilterType {
    fn from(filter: Filter) -> FilterType {
        match filter {
            Filter::Nearest => FilterType::Nearest,
            Filter::Triangle => FilterType::Triangle,
            Filter::CatmullRom => FilterType::CatmullRom,
            Filter::Gaussian => FilterType::Gaussian,
            Filter::Lanczos3 => FilterType::Lanczos3,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ProcessWrapper;
    use {
        async_std::task::block_on,
        std::{fs::read_dir, path::PathBuf, sync::Arc},
    };

    fn config() -> super::ProcessConfig {
        super::ProcessConfig {
            size: 50,
            k: 1,
            hamerly: false,
            calc_unit: crate::CalculationUnit::Average,
            color_space: crate::ColorSpace::CIELAB,
            dist_algo: crate::DistanceAlgorithm::CIEDE2000,
            filter: super::Filter::Nearest,
        }
    }

    fn lib() -> Vec<PathBuf> {
        read_dir("../image_crawler/test")
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
            .collect::<Vec<_>>()
    }

    fn process(mut proc: ProcessWrapper) {
        let library = lib();

        block_on(async move {
            let mut lib = Vec::with_capacity(library.len());
            let tasks = proc.index(library);
            for task in tasks {
                if let Some(i) = task.await {
                    lib.push(i);
                }
            }
            let lib = Arc::new(lib);

            proc.pre_fill(lib);
            while let Some(tasks) = proc.fill() {
                for task in tasks {
                    let (mask, replace) = task.await;
                    proc.post_fill_step(mask, replace);
                }
                proc.post_fill();
            }
        });
    }

    #[test]
    fn image_process() {
        let config = config();
        let (proc, _) = super::ProcessWrapper::new(
            config,
            "../static/images/testdata.jpg".to_string(),
            "test.png".to_string(),
            false,
        );

        process(proc);
    }

    #[test]
    fn video_process() {
        crate::init().unwrap();
        ffmpeg::log::set_level(ffmpeg::log::Level::Info);

        let config = config();
        let (proc, _) = super::ProcessWrapper::new(
            config,
            "../static/videos/testdata.mp4".to_string(),
            "test.mp4".to_string(),
            true,
        );

        process(proc);
    }
}

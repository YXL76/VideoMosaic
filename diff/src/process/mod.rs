mod average;
mod kmeans;
mod pixel;

use {
    crate::{
        ciede2000, converter, CalculationUnit, ColorSpace, DistanceAlgorithm, F32Wrapper,
        FrameIter, ImageDump, MyHsv, MySrgb, RawColor, Transcode, Variance,
    },
    async_std::task::{spawn_blocking, JoinHandle},
    average::AverageImpl,
    futures::stream::{futures_unordered, FuturesUnordered},
    image::{imageops::FilterType, DynamicImage, GenericImageView, ImageBuffer, Pixel, RgbImage},
    kmeans::KMeansImpl,
    palette::{Lab, Pixel as Palette_Pixel},
    pixel::PixelImpl,
    std::{collections::BTreeMap, fmt, path::PathBuf, sync::Arc},
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
    frames: i64,
    width: u32,
    height: u32,
    quad_iter: Option<usize>,
    overlay: Option<u8>,
    lib: Option<Arc<Vec<LibItem>>>,
    masks: Option<Vec<Mask>>,
    next: Option<Arc<RgbImage>>,
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
            quad_iter,
            overlay,
        }: ProcessConfig,
        input: String,
        output: String,
        video: bool,
    ) -> Self {
        let size = size as u32;
        let filter = filter.into();

        let distance = Box::new(match dist_algo {
            DistanceAlgorithm::Euclidean => match color_space {
                ColorSpace::HSV => |a: &RawColor, b: &RawColor| {
                    let a = MyHsv::from_raw(a);
                    let b = MyHsv::from_raw(b);
                    (a.hue.to_positive_degrees() - b.hue.to_positive_degrees()).powi(2)
                        + ((a.saturation - b.saturation) * 360.).powi(2)
                        + ((a.value - b.value) * 360.).powi(2)
                },
                _ => |a: &RawColor, b: &RawColor| {
                    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
                },
            },
            DistanceAlgorithm::CIEDE2000 => match color_space {
                ColorSpace::RGB => ciede2000::<MySrgb>,
                ColorSpace::HSV => ciede2000::<MyHsv>,
                ColorSpace::CIELAB => ciede2000::<Lab>,
            },
        });

        let converter = Box::new(match color_space {
            ColorSpace::RGB => converter::<MySrgb>,
            ColorSpace::HSV => converter::<MyHsv>,
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

        let (iter, frames, width, height) = if video {
            let (iter, frames, width, height) = Transcode::new(input, output);
            let iter: Box<dyn FrameIter + Sync + Send + 'static> = Box::new(iter);
            (iter, frames, width, height)
        } else {
            let (iter, frames, width, height) = ImageDump::new(input, output);
            let iter: Box<dyn FrameIter + Sync + Send + 'static> = Box::new(iter);
            (iter, frames, width, height)
        };

        Self {
            iter,
            inner,
            buf: ImageBuffer::new(width, height),
            frames,
            width,
            height,
            quad_iter,
            overlay,
            lib: None,
            masks: None,
            next: None,
        }
    }

    #[inline(always)]
    pub fn frames(&self) -> i64 {
        self.frames
    }

    #[inline(always)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline(always)]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline(always)]
    fn inner(&self) -> Arc<dyn Process + Sync + Send + 'static> {
        self.inner.clone()
    }

    #[inline(always)]
    pub fn index(&self, libraries: Vec<PathBuf>) -> Tasks<Option<LibItem>> {
        let (nwidth, nheight) = match self.quad_iter {
            Some(iterations) => {
                // 1 + 4 + 16 + ...
                let min_depth = (iterations * 3 + 1).log2() / 2 + 1;
                let guess = min_depth * 2;
                (self.width / guess, self.height / guess)
            }
            None => (self.inner.size(), self.inner.size()),
        };
        libraries
            .into_iter()
            .map(|lib| {
                let inner = self.inner();
                spawn_blocking(move || {
                    if let Ok(img) = image::open(lib) {
                        let img = img
                            .resize_to_fill(nwidth, nheight, inner.filter())
                            .into_rgb8();
                        return Some(inner.index_step(img));
                    }
                    None
                })
            })
            .collect::<FuturesUnordered<_>>()
    }

    #[inline(always)]
    pub fn post_index(&mut self, lib: Arc<Vec<LibItem>>) {
        self.lib = Some(lib);
    }

    #[inline(always)]
    pub fn pre_fill(&mut self) -> bool {
        self.next = self.iter.next();
        if self.next.is_none() {
            self.lib = None;
            self.masks = None;
            self.iter.flush();
            return false;
        }

        if let Some(iterations) = self.quad_iter {
            let img = self.next.as_ref().unwrap();
            let mut heap: BTreeMap<F32Wrapper, Mask> = BTreeMap::new();
            heap.insert(F32Wrapper(0.), (0, 0, self.width, self.height));
            for _ in 0..iterations {
                let (_, (x, y, w, h)) = heap.pop_last().unwrap();
                if w <= 4 || h <= 4 {
                    heap.insert(F32Wrapper(0.), (x, y, w, h));
                    continue;
                }

                let w2 = w / 2;
                let h2 = h / 2;
                let xm = x + w2;
                let ym = y + h2;
                let quad = [
                    (x, y, w2, h2),
                    (xm, y, w - w2, h2),
                    (x, ym, w2, h - h2),
                    (xm, ym, w - w2, h - h2),
                ];

                for (x, y, w, h) in quad {
                    let mut rgb = [Variance::new(); 3];
                    for j in y..(y + h) {
                        for i in x..(x + w) {
                            let raw = &img.get_pixel(i, j).0;
                            for (idx, &raw) in raw.into_iter().enumerate() {
                                rgb[idx].next(raw as i64);
                            }
                        }
                    }

                    const FACTOR: [f32; 3] = [0.299, 0.587, 0.114];
                    let error = rgb
                        .into_iter()
                        .enumerate()
                        .map(|(idx, part)| part.variance() * FACTOR[idx])
                        .sum::<f32>();
                    heap.insert(F32Wrapper(error), (x, y, w, h));
                }
            }

            self.masks = Some(heap.into_values().collect());
        } else {
            if self.masks.is_some() {
                return true;
            }

            let size = self.inner.size();
            let mut masks = Vec::with_capacity(
                (((self.width / size) + 1) * ((self.height / size) + 1)) as usize,
            );
            for y in (0..self.height).step_by(size as usize) {
                for x in (0..self.width).step_by(size as usize) {
                    let w = size.min(self.width - x);
                    let h = size.min(self.height - y);
                    masks.push((x, y, w, h));
                }
            }
            self.masks = Some(masks);
        };

        true
    }

    #[inline(always)]
    pub fn fill(&mut self) -> Tasks<(Mask, Arc<RgbImage>)> {
        let lib = self.lib.as_ref().unwrap();
        let img = self.next.as_ref().unwrap();
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
            .collect::<FuturesUnordered<_>>()
    }

    #[inline(always)]
    pub fn post_fill_step(&mut self, (x, y, w, h): Mask, mut replace: Arc<RgbImage>) {
        if replace.width() != w || replace.height() != h {
            replace = Arc::new(
                DynamicImage::ImageRgb8(replace.inner().clone())
                    .resize_to_fill(w, h, self.inner.filter())
                    .into_rgb8(),
            );
        }
        for j in 0..h {
            for i in 0..w {
                let p = replace.get_pixel(i, j);
                self.buf.put_pixel(i + x, j + y, *p);
            }
        }
    }

    /// See [`overlay`](#image::imageops::overlay)
    #[inline(always)]
    pub fn post_fill(&mut self) {
        if let Some(bottom_alpha) = self.overlay {
            let top_alpha = u8::MAX - bottom_alpha;
            let img = self.next.as_ref().unwrap();
            for j in 0..self.height {
                for i in 0..self.width {
                    let mut top = self.buf.get_pixel(i, j).to_rgba();
                    let mut bottom = img.get_pixel(i, j).to_rgba();
                    top.0[3] = top_alpha;
                    bottom.0[3] = bottom_alpha;
                    bottom.blend(&top);
                    self.buf.put_pixel(i, j, bottom.to_rgb());
                }
            }
        }
        self.iter.post_next(&self.buf)
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
    pub quad_iter: Option<usize>,
    pub overlay: Option<u8>,
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
            quad_iter: Default::default(),
            overlay: Default::default(),
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
    use {
        crate::{ProcessWrapper, IMAGE_FILTER},
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
            quad_iter: Some(1000),
            overlay: Some(127),
        }
    }

    fn lib() -> Vec<PathBuf> {
        read_dir("../crawler/test")
            .unwrap()
            .filter_map(|res| match res.as_ref() {
                Ok(entry) => {
                    let path = entry.path();
                    let ext = path
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default();
                    if path.is_file() && IMAGE_FILTER.contains(&ext) {
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
            proc.post_index(Arc::new(lib));

            while proc.pre_fill() {
                let tasks = proc.fill();
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
        let proc = super::ProcessWrapper::new(
            config,
            "../static/images/testdata.jpg".to_string(),
            "test.png".to_string(),
            false,
        );

        process(proc);
    }

    #[test]
    fn video_process() {
        crate::init();
        ffmpeg::log::set_level(ffmpeg::log::Level::Info);

        let config = config();
        let proc = super::ProcessWrapper::new(
            config,
            "../static/videos/testdata.mp4".to_string(),
            "test.mp4".to_string(),
            true,
        );

        process(proc);
    }
}

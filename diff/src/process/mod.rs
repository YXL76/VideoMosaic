mod average;
mod k_means;
mod pixel;

use {
    crate::{
        ciede2000, converter, CalculationUnit, ColorSpace, DistanceAlgorithm, F32Wrapper,
        FrameIter, ImageDump, MyHsv, MySrgb, RawColor, Transcode, Variance,
    },
    async_std::task::{spawn_blocking, JoinHandle},
    average::AverageImpl,
    futures::stream::{futures_unordered, FuturesUnordered},
    image::{
        imageops::{crop, resize, FilterType},
        GenericImageView, ImageBuffer, Pixel, RgbImage,
    },
    k_means::KMeansImpl,
    palette::{Lab, Pixel as PalettePixel},
    pixel::PixelImpl,
    std::{borrow::Cow, collections::BTreeMap, fmt, path::PathBuf, sync::Arc},
};

pub type Mask = (u32, u32, u32, u32);
type Task<T> = JoinHandle<T>;
type Tasks<T> = FuturesUnordered<Task<T>>;
pub type TasksIter<T> = futures_unordered::IntoIter<Task<T>>;
pub type LibItem = (RawColor, RgbImage);

type Converter = Box<dyn Fn(&[u8]) -> RawColor + Sync + Send>;
type Distance = Box<dyn Fn(&RawColor, &RawColor) -> f32 + Sync + Send>;

trait Process {
    fn size(&self) -> u32;

    fn prev(&self) -> &Option<RgbImage>;

    fn next(&self) -> &Option<RgbImage>;

    fn prev_mut(&mut self) -> &mut Option<RgbImage>;

    fn next_mut(&mut self) -> &mut Option<RgbImage>;

    fn set_lib(&mut self, lib_color: Vec<RawColor>, lib_image: Vec<RgbImage>);

    fn get_image(&self, idx: usize) -> &RgbImage;

    fn filter(&self) -> FilterType;

    fn index_step(&self, img: RgbImage) -> LibItem;

    fn fill_step(&self, mask: Mask) -> (Mask, usize);
}

// TODO: Quad cannot work with pre-calc color

pub struct ProcessWrapper {
    iter: Box<dyn FrameIter + Sync + Send + 'static>,
    inner: Arc<dyn Process + Sync + Send + 'static>,
    buf: RgbImage,
    composite: Option<RgbImage>,
    frames: i64,
    width: u32,
    height: u32,
    quad_iter: Option<usize>,
    overlay: Option<u8>,
    masks: Box<[Mask]>,
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

        let composite = overlay.map(|_| ImageBuffer::new(width, height));

        Self {
            iter,
            inner,
            buf: ImageBuffer::new(width, height),
            composite,
            frames,
            width,
            height,
            quad_iter,
            overlay,
            masks: Vec::new().into_boxed_slice(),
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
                let inner = self.inner.clone();
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
    pub fn post_index(&mut self, lib_color: Vec<RawColor>, lib_image: Vec<RgbImage>) {
        Arc::get_mut(&mut self.inner)
            .unwrap()
            .set_lib(lib_color, lib_image);
    }

    #[inline(always)]
    pub fn pre_fill(&mut self) -> bool {
        {
            let inner = Arc::get_mut(&mut self.inner).unwrap();
            if self.quad_iter.is_none() {
                *inner.prev_mut() = inner.next_mut().take();
            }
            let next = inner.next_mut();
            *next = self.iter.next();
            if next.is_none() {
                self.flush();
                return false;
            }
        }

        if let Some(iterations) = self.quad_iter {
            let next = self.inner.next().as_ref().unwrap();
            let mut heap: BTreeMap<F32Wrapper, Mask> = BTreeMap::new();
            heap.insert(F32Wrapper(0.), (0, 0, self.width, self.height));
            for _ in 0..iterations {
                let (_, (x, y, w, h)) = heap.pop_last().unwrap();
                if w <= 8 || h <= 8 {
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
                            let raw = next.get_pixel(i, j).0;
                            for (idx, raw) in raw.into_iter().enumerate() {
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

            self.masks = heap.into_values().collect();
        } else {
            if self.inner.prev().is_some() {
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
            self.masks = masks.into_boxed_slice();
        };

        true
    }

    #[inline(always)]
    pub fn fill(&mut self) -> Tasks<(Mask, usize)> {
        let masks = self.masks.iter();
        let masks: Box<dyn Iterator<Item = &Mask>> =
            match self.quad_iter.is_none() && self.inner.prev().is_some() {
                true => {
                    let prev = self.inner.prev().as_ref().unwrap();
                    let next = self.inner.next().as_ref().unwrap();
                    Box::new(masks.filter(|mask| {
                        let mut total: usize = 0;
                        let mut count: usize = 0;
                        const STEP: usize = 5;
                        for j in (mask.1..(mask.1 + mask.3)).step_by(STEP) {
                            for i in (mask.0..(mask.0 + mask.2)).step_by(STEP) {
                                let prev_pixel = prev.get_pixel(i, j);
                                let next_pixel = next.get_pixel(i, j);
                                total += 1;
                                if prev_pixel != next_pixel {
                                    count += 1;
                                }
                            }
                        }
                        count * 2 >= total
                    }))
                }
                false => Box::new(masks),
            };
        masks
            .map(|&mask| {
                let inner = self.inner.clone();
                spawn_blocking(move || inner.fill_step(mask))
            })
            .collect::<FuturesUnordered<_>>()
    }

    #[inline(always)]
    pub fn post_fill_step(&mut self, (x, y, w, h): Mask, replace_idx: usize) {
        let mut replace = Cow::Borrowed(self.inner.get_image(replace_idx));
        if replace.width() != w || replace.height() != h {
            let width = replace.width();
            let height = replace.height();
            let nwidth = w;
            let nheight = h;

            // See [`resize_dimensions`](#image::math::utils::resize_dimensions)
            let (width2, height2) = {
                let ratio = u64::from(width) * u64::from(nheight);
                let nratio = u64::from(nwidth) * u64::from(height);

                let use_width = nratio > ratio;
                let intermediate = if use_width {
                    u64::from(height) * u64::from(nwidth) / u64::from(width)
                } else {
                    u64::from(width) * u64::from(nheight) / u64::from(height)
                };
                let intermediate = std::cmp::max(1, intermediate);
                if use_width {
                    if intermediate <= u64::from(u32::MAX) {
                        (nwidth, intermediate as u32)
                    } else {
                        (
                            (u64::from(nwidth) * u64::from(u32::MAX) / intermediate) as u32,
                            u32::MAX,
                        )
                    }
                } else if intermediate <= u64::from(u32::MAX) {
                    (intermediate as u32, nheight)
                } else {
                    (
                        u32::MAX,
                        (u64::from(nheight) * u64::from(u32::MAX) / intermediate) as u32,
                    )
                }
            };

            // See [`resize_to_fill`](#image::DynamicImage::resize_to_fill)
            let mut intermediate = resize(replace.inner(), width2, height2, self.inner.filter());
            let (iwidth, iheight) = intermediate.dimensions();
            let ratio = u64::from(iwidth) * u64::from(nheight);
            let nratio = u64::from(nwidth) * u64::from(iheight);

            let intermediate = if nratio > ratio {
                let y = (iheight - nheight) / 2;
                crop(&mut intermediate, 0, y, nwidth, nheight)
            } else {
                crop(&mut intermediate, (iwidth - nwidth) / 2, 0, nwidth, nheight)
            };

            replace = Cow::Owned(intermediate.to_image());
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
            let img = self.inner.next().as_ref().unwrap();
            let composite = self.composite.as_mut().unwrap();
            for j in 0..self.height {
                for i in 0..self.width {
                    let mut top = self.buf.get_pixel(i, j).to_rgba();
                    let mut bottom = img.get_pixel(i, j).to_rgba();
                    top.0[3] = top_alpha;
                    bottom.0[3] = bottom_alpha;
                    bottom.blend(&top);
                    composite.put_pixel(i, j, bottom.to_rgb());
                }
            }
            self.iter.post_next(composite)
        } else {
            self.iter.post_next(&self.buf)
        };
    }

    #[inline(always)]
    pub fn flush(&mut self) {
        self.iter.flush();

        // to be dropped automatic
        // self.composite = None;
        // self.quad_iter = None;
        // self.overlay = None;
        // self.lib = None;
        // self.masks = None;
        // self.prev = None;
        // self.next = None;
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
        std::{fs::read_dir, path::PathBuf},
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
            quad_iter: None,
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
            let mut lib_color = Vec::with_capacity(library.len());
            let mut lib_image = Vec::with_capacity(library.len());
            let tasks = proc.index(library);
            for task in tasks {
                if let Some((color, image)) = task.await {
                    lib_color.push(color);
                    lib_image.push(image);
                }
            }
            proc.post_index(lib_color, lib_image);

            while proc.pre_fill() {
                let tasks = proc.fill();
                for task in tasks {
                    let (mask, replace_idx) = task.await;
                    proc.post_fill_step(mask, replace_idx);
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

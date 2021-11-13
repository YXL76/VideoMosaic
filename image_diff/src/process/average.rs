use {
    super::{Process, ProcessConfig, ProcessResult},
    crate::ColorSpace,
    image::{self, imageops::FilterType, ImageBuffer, RgbImage},
    palette::{Hsv, IntoColor, Lab, Pixel as PalettePixel, Srgb},
    parking_lot::Mutex,
    rayon::prelude::*,
    std::path::PathBuf,
};

type Lib = Vec<([f64; 3], Box<RgbImage>)>;

pub struct AverageProcImpl {
    size: u32,
    converter: Box<dyn Fn(&[u8; 3]) -> [f64; 3] + Sync + Send>,
}

impl Process for AverageProcImpl {
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<&str> {
        let lib = self.index(library)?;
        self.fill(target, lib)?;
        Ok("")
    }
}

impl AverageProcImpl {
    pub fn new(config: ProcessConfig) -> Self {
        let ProcessConfig {
            size, color_space, ..
        } = config;

        let converter = Box::new(match color_space {
            ColorSpace::RGB => |rgb: &[u8; 3]| Srgb::from_raw(rgb).into_format::<f64>().into_raw(),
            ColorSpace::HSV => |rgb: &[u8; 3]| {
                let hsv: Hsv<_, f64> = Srgb::from_raw(rgb).into_format::<f64>().into_color();
                hsv.into_raw()
            },
            ColorSpace::CIEXYZ => |rgb: &[u8; 3]| {
                let lab: Lab<_, f64> = Srgb::from_raw(rgb).into_format::<f64>().into_color();
                lab.into_raw()
            },
        });

        Self { size, converter }
    }

    fn index(&self, libraries: &[PathBuf]) -> ProcessResult<Lib> {
        let vec = Mutex::new(Vec::with_capacity(libraries.len()));
        libraries.into_par_iter().for_each(|lib| {
            if let Ok(img) = image::open(lib) {
                let img = img
                    .resize_to_fill(self.size, self.size, FilterType::Nearest)
                    .into_rgb8();
                vec.lock().push((
                    self.average_color(&img, 0, 0, self.size, self.size),
                    Box::new(img),
                ));
            }
        });
        let vec = vec.into_inner();
        if vec.len() == 0 {
            return Err("");
        }
        Ok(vec)
    }

    fn fill(&self, target: &PathBuf, lib: Lib) -> ProcessResult<&str> {
        let img = image::open(target).unwrap().into_rgb8();
        let (width, height) = img.dimensions();
        let imgbuf = Mutex::new(ImageBuffer::new(width, height));

        for y in (0..height).step_by(self.size as usize) {
            (0..width)
                .into_par_iter()
                .step_by(self.size as usize)
                .for_each(|x| {
                    let w = self.size.min(width - x);
                    let h = self.size.min(height - y);
                    let raw = self.average_color(&img, x, y, w, h);

                    let (_, replace) = lib
                        .par_iter()
                        .min_by(|(a, _), (b, _)| {
                            square_distance(a, &raw)
                                .partial_cmp(&square_distance(b, &raw))
                                .unwrap()
                        })
                        .unwrap();

                    {
                        let mut guard = imgbuf.lock();
                        for j in 0..h {
                            for i in 0..w {
                                let p = replace.get_pixel(i, j);
                                guard.put_pixel(i + x, j + y, *p);
                            }
                        }
                    }
                })
        }

        imgbuf
            .into_inner()
            .save("/home/yxl/MosaicVideo/tmp.png")
            .unwrap();

        Ok("")
    }

    fn average_color(&self, img: &RgbImage, x: u32, y: u32, w: u32, h: u32) -> [f64; 3] {
        let mut ans = [0f64; 3];
        for j in y..(y + h) {
            for i in x..(x + w) {
                let raw = (self.converter)(&img.get_pixel(i, j).0);
                ans[0] += raw[0];
                ans[1] += raw[1];
                ans[2] += raw[2];
            }
        }
        let count = (w * h) as f64;
        ans[0] /= count;
        ans[1] /= count;
        ans[2] /= count;
        ans
    }
}

#[inline(always)]
fn square_distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

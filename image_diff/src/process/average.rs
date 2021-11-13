use {
    super::{Process, ProcessConfig, ProcessResult},
    crate::ColorSpace,
    image::{self, imageops::FilterType, GenericImage, ImageBuffer, RgbImage},
    palette::{Hsv, IntoColor, Lab, Pixel as PalettePixel, Srgb},
    std::{collections::HashMap, path::PathBuf},
};

pub struct AverageProcImpl {
    size: u32,
    converter: Box<dyn Fn(&[u8; 3]) -> [f64; 3]>,
}

impl Process for AverageProcImpl {
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<&str> {
        let lib = self.index_lib(library)?;
        self.fill_result(target, lib)?;
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

    fn index_lib<'a>(
        &self,
        libraries: &'a [PathBuf],
    ) -> ProcessResult<HashMap<&'a PathBuf, [f64; 3]>> {
        let mut map = HashMap::new();
        for lib in libraries {
            if let Ok(img) = image::open(lib) {
                let img = img
                    .resize_to_fill(self.size, self.size, FilterType::Nearest)
                    .into_rgb8();
                map.insert(lib, self.average_color(&img, 0, 0, self.size, self.size));
            }
        }
        if map.len() == 0 {
            return Err("");
        }
        Ok(map)
    }

    fn fill_result(
        &self,
        target: &PathBuf,
        lib: HashMap<&PathBuf, [f64; 3]>,
    ) -> ProcessResult<&str> {
        let img = image::open(target).unwrap().into_rgb8();
        let (width, height) = img.dimensions();
        let mut imgbuf = ImageBuffer::new(width, height);

        for x in (0..width).step_by(self.size as usize) {
            for y in (0..height).step_by(self.size as usize) {
                let w = self.size.min(width - x);
                let h = self.size.min(height - y);
                let raw = self.average_color(&img, x, y, w, h);
                /* let lab: Lab<_, f64> = Srgb::from_raw(&rgb)
                .into_format::<f64>()
                .into_linear()
                .into_color(); */

                let (path, _) = lib
                    .iter()
                    .min_by(|(_, a), (_, b)| {
                        square_distance(a, &raw)
                            .partial_cmp(&square_distance(b, &raw))
                            .unwrap()
                    })
                    .unwrap();

                let replace = image::open(path)
                    .unwrap()
                    .resize_to_fill(w, h, FilterType::Nearest);

                imgbuf.copy_from(&replace, x, y).unwrap();
            }
        }
        Ok("")
    }

    fn average_color(&self, img: &RgbImage, x: u32, y: u32, w: u32, h: u32) -> [f64; 3] {
        let mut ans = [0f64; 3];
        for i in x..(x + w) {
            for j in y..(y + h) {
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

#[inline]
fn square_distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

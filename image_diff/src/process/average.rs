use {
    super::{Converter, Distance, Process, ProcessResult},
    image::{self, imageops::FilterType, ImageBuffer, RgbImage},
    parking_lot::Mutex,
    rayon::prelude::*,
    std::path::PathBuf,
};

type Lib = Vec<([f64; 3], Box<RgbImage>)>;

pub struct AverageProcImpl {
    size: u32,
    converter: Converter,
    distance: Distance,
}

impl Process for AverageProcImpl {
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<&str> {
        let lib = self.index(library)?;
        self.fill(target, lib)?;
        Ok("")
    }
}

impl AverageProcImpl {
    pub fn new(size: u32, converter: Converter, distance: Distance) -> Self {
        Self {
            size,
            converter,
            distance,
        }
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
                            (self.distance)(a, &raw)
                                .partial_cmp(&(self.distance)(b, &raw))
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

use {
    super::{Converter, Distance, Process, ProcessResult, ProcessStep, RawColor},
    image::{self, imageops::FilterType, ImageBuffer, RgbImage},
    parking_lot::Mutex,
    rayon::prelude::*,
    std::path::PathBuf,
};

type ImgData = Vec<RawColor>;

pub struct PixelProcImpl {
    size: u32,
    converter: Converter,
    distance: Distance,
}

impl Process for PixelProcImpl {
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage> {
        self.fill(target, self.index(library)?)
    }
}

impl ProcessStep for PixelProcImpl {
    type Lib = Vec<(ImgData, Box<RgbImage>)>;

    fn index(&self, libraries: &[PathBuf]) -> ProcessResult<Self::Lib> {
        let Self {
            size, converter, ..
        } = self;

        let vec = Mutex::new(Vec::with_capacity(libraries.len()));
        libraries.into_par_iter().for_each(|lib| {
            if let Ok(img) = image::open(lib) {
                let img = img
                    .resize_to_fill(*size, *size, FilterType::Nearest)
                    .into_rgb8();
                let mut buf: ImgData = Vec::with_capacity((size * size) as usize);
                for j in 0..*size {
                    for i in 0..*size {
                        buf.push(converter(&img.get_pixel(i, j).0))
                    }
                }
                vec.lock().push((buf, Box::new(img)));
            }
        });
        let vec = vec.into_inner();
        if vec.len() == 0 {
            return Err("");
        }
        Ok(vec)
    }

    fn fill(&self, target: &PathBuf, lib: Self::Lib) -> ProcessResult<RgbImage> {
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

                    let (_, replace) = lib
                        .par_iter()
                        .min_by(|(a, _), (b, _)| {
                            self.compare(&img, a, x, y, w, h)
                                .partial_cmp(&self.compare(&img, b, x, y, w, h))
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

        Ok(imgbuf.into_inner())
    }
}

impl PixelProcImpl {
    pub fn new(size: u32, converter: Converter, distance: Distance) -> Self {
        Self {
            size,
            converter,
            distance,
        }
    }

    fn compare(&self, img: &RgbImage, other: &ImgData, x: u32, y: u32, w: u32, h: u32) -> f32 {
        let Self {
            size,
            converter,
            distance,
            ..
        } = self;

        let mut ans = 0f32;
        for j in 0..h {
            for i in 0..w {
                ans += distance(
                    &converter(&img.get_pixel(i + x, j + y).0),
                    &other[(j * size + i) as usize],
                );
            }
        }
        ans
    }
}

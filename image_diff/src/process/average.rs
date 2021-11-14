use {
    super::{Converter, Distance, Process, ProcessResult, ProcessStep, RawColor},
    image::{self, RgbImage},
    parking_lot::Mutex,
    rayon::prelude::*,
    std::path::PathBuf,
};

pub struct AverageProcImpl {
    size: u32,
    converter: Converter,
    distance: Distance,
}

impl Process for AverageProcImpl {
    #[inline(always)]
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage> {
        self.fill(target, self.index(library)?)
    }
}

impl ProcessStep for AverageProcImpl {
    type Item = (RawColor, Box<RgbImage>);

    #[inline(always)]
    fn size(&self) -> u32 {
        self.size
    }

    #[inline(always)]
    fn index_step(&self, img: RgbImage) -> Self::Item {
        (
            self.average(&img, 0, 0, self.size, self.size),
            Box::new(img),
        )
    }

    #[inline(always)]
    fn fill_step(
        &self,
        img: &RgbImage,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        lib: &Vec<Self::Item>,
        buf: &Mutex<RgbImage>,
    ) {
        let raw = self.average(img, x, y, w, h);

        let (_, replace) = lib
            .par_iter()
            .min_by(|(a, _), (b, _)| {
                (self.distance)(a, &raw)
                    .partial_cmp(&(self.distance)(b, &raw))
                    .unwrap()
            })
            .unwrap();

        {
            let mut guard = buf.lock();
            for j in 0..h {
                for i in 0..w {
                    let p = replace.get_pixel(i, j);
                    guard.put_pixel(i + x, j + y, *p);
                }
            }
        }
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

    #[inline(always)]
    fn average(&self, img: &RgbImage, x: u32, y: u32, w: u32, h: u32) -> RawColor {
        let Self { converter, .. } = self;
        let mut ans = [0f32; 3];
        for j in y..(y + h) {
            for i in x..(x + w) {
                let raw = converter(&img.get_pixel(i, j).0);
                ans[0] += raw[0];
                ans[1] += raw[1];
                ans[2] += raw[2];
            }
        }
        let count = (w * h) as f32;
        ans[0] /= count;
        ans[1] /= count;
        ans[2] /= count;
        ans
    }
}

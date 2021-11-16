use {
    super::{Distance, Process, ProcessResult, ProcessStep},
    crate::utils::{Color, RawColor},
    image::{self, RgbImage},
    parking_lot::Mutex,
    rayon::prelude::*,
    std::{marker::PhantomData, path::PathBuf},
};

pub struct AverageProc<T: Color> {
    size: u32,
    distance: Distance,
    color: PhantomData<T>,
}

impl<T: Color> Process for AverageProc<T> {
    #[inline(always)]
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage> {
        self.do_run(target, library)
    }
}

impl<T: Color> ProcessStep<T> for AverageProc<T> {
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
        let Self { distance, .. } = self;
        let raw = self.average(img, x, y, w, h);

        let (_, replace) = lib
            .par_iter()
            .min_by(|(a, _), (b, _)| distance(a, &raw).partial_cmp(&distance(b, &raw)).unwrap())
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

impl<T: Color> AverageProc<T> {
    pub fn new(size: u32, distance: Distance) -> Self {
        Self {
            size,
            distance,
            color: PhantomData::default(),
        }
    }

    // #[inline(always)]
    fn average(&self, img: &RgbImage, x: u32, y: u32, w: u32, h: u32) -> RawColor {
        let mut ans = [0f32; 3];
        for j in y..(y + h) {
            for i in x..(x + w) {
                let raw = Self::converter(&img.get_pixel(i, j).0);
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

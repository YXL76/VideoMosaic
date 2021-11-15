use {
    super::{Color, Distance, Process, ProcessResult, ProcessStep, RawColor},
    image::{self, RgbImage},
    parking_lot::Mutex,
    rayon::prelude::*,
    std::{marker::PhantomData, path::PathBuf},
};

type ImgData = Vec<RawColor>;

pub struct PixelProc<T: Color> {
    size: u32,
    distance: Distance,
    color: PhantomData<T>,
}

impl<T: Color> Process for PixelProc<T> {
    #[inline(always)]
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage> {
        self.fill(target, self.index(library)?)
    }
}

impl<T: Color> ProcessStep<T> for PixelProc<T> {
    type Item = (ImgData, Box<RgbImage>);

    #[inline(always)]
    fn size(&self) -> u32 {
        self.size
    }

    #[inline(always)]
    fn index_step(&self, img: RgbImage) -> Self::Item {
        let Self { size, .. } = self;
        let mut buf: ImgData = Vec::with_capacity((size * size) as usize);
        for j in 0..*size {
            for i in 0..*size {
                buf.push(Self::converter(&img.get_pixel(i, j).0))
            }
        }
        (buf, Box::new(img))
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
        let (_, replace) = lib
            .par_iter()
            .min_by(|(a, _), (b, _)| {
                self.compare(img, a, x, y, w, h)
                    .partial_cmp(&self.compare(img, b, x, y, w, h))
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

impl<T: Color> PixelProc<T> {
    pub fn new(size: u32, distance: Distance) -> Self {
        Self {
            size,
            distance,
            color: PhantomData::default(),
        }
    }

    // #[inline(always)]
    fn compare(&self, img: &RgbImage, other: &ImgData, x: u32, y: u32, w: u32, h: u32) -> f32 {
        let Self { size, distance, .. } = self;

        let mut ans = 0f32;
        for j in 0..h {
            for i in 0..w {
                ans += distance(
                    &Self::converter(&img.get_pixel(i + x, j + y).0),
                    &other[(j * size + i) as usize],
                );
            }
        }
        ans
    }
}

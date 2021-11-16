use {
    super::{ColorSpace, Distance, Process, ProcessResult, ProcessStep},
    crate::utils::{Color, RawColor},
    image::{self, RgbImage},
    kmeans_colors::{get_kmeans, Kmeans},
    palette::{IntoColor, Pixel, Srgb},
    parking_lot::Mutex,
    rayon::prelude::*,
    std::{marker::PhantomData, path::PathBuf},
};

pub struct KMeansProc<T: Color> {
    size: u32,
    converge: f32,
    max_iter: usize,
    distance: Distance,
    color: PhantomData<T>,
}

impl<T: Color> Process for KMeansProc<T> {
    #[inline(always)]
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage> {
        self.do_run(target, library)
    }
}

impl<T: Color> ProcessStep<T> for KMeansProc<T> {
    type Item = (Vec<RawColor>, Box<RgbImage>);

    #[inline(always)]
    fn size(&self) -> u32 {
        self.size
    }

    #[inline(always)]
    fn index_step(&self, img: RgbImage) -> Self::Item {
        (
            self.k_means(&img, 0, 0, self.size, self.size),
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
        let raw = self.k_means(img, x, y, w, h);

        let (_, replace) = lib
            .par_iter()
            .min_by(|(a, _), (b, _)| {
                distance(&a[0], &raw[0])
                    .partial_cmp(&distance(&b[0], &raw[0]))
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

impl<T: Color> KMeansProc<T> {
    const RUNS: u64 = 3;
    const FACTOR_RGB: f32 = 0.0025;
    const FACTOR_LAB: f32 = 10.;
    const MAX_ITER_RGB: usize = 10;
    const MAX_ITER_LAB: usize = 20;

    pub fn new(size: u32, distance: Distance, color_space: ColorSpace) -> Self {
        let converge = match color_space {
            ColorSpace::RGB | ColorSpace::HSV => Self::FACTOR_RGB,
            ColorSpace::CIELAB => Self::FACTOR_LAB,
        };

        let max_iter = match color_space {
            ColorSpace::RGB | ColorSpace::HSV => Self::MAX_ITER_RGB,
            ColorSpace::CIELAB => Self::MAX_ITER_LAB,
        };

        Self {
            size,
            converge,
            max_iter,
            distance,
            color: PhantomData::default(),
        }
    }

    // #[inline(always)]
    fn k_means(&self, img: &RgbImage, x: u32, y: u32, w: u32, h: u32) -> Vec<RawColor> {
        let Self {
            max_iter, converge, ..
        } = self;
        let mut buf: Vec<T> = Vec::with_capacity((w * h) as usize);
        for j in y..(y + h) {
            for i in x..(x + w) {
                let color = Srgb::from_raw(&img.get_pixel(i, j).0)
                    .into_format::<f32>()
                    .into_color();
                buf.push(color)
            }
        }
        let res = (0..Self::RUNS).fold(Kmeans::new(), |res, i| {
            let run_res = get_kmeans(1, *max_iter, *converge, false, &buf, i);
            if run_res.score < res.score {
                run_res
            } else {
                res
            }
        });
        let mut res = T::sort_indexed_colors(&res.centroids, &res.indices);
        res.sort_unstable_by(|a, b| (b.percentage).partial_cmp(&a.percentage).unwrap());
        res.iter().map(|i| i.centroid.into_raw()).collect()
    }
}

use {
    super::{ColorSpace, Distance, KMeansResult, Process, ProcessResult, ProcessStep, RawColor},
    image::{self, RgbImage},
    kmeans_colors::{get_kmeans, Calculate, Kmeans, Sort},
    palette::{convert::FromColorUnclamped, Clamp, IntoColor, Lab, Pixel, Srgb},
    parking_lot::Mutex,
    rayon::prelude::*,
    std::path::PathBuf,
};

pub struct KMeansProcImpl {
    size: u32,
    converge: f32,
    max_iter: usize,
    distance: Distance,
    k_means: KMeansResult,
}

impl Process for KMeansProcImpl {
    #[inline(always)]
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage> {
        self.fill(target, self.index(library)?)
    }
}

impl ProcessStep for KMeansProcImpl {
    type Item = (Vec<RawColor>, Box<RgbImage>);

    #[inline(always)]
    fn size(&self) -> u32 {
        self.size
    }

    #[inline(always)]
    fn index_step(&self, img: RgbImage) -> Self::Item {
        let Self { k_means, .. } = self;
        (
            k_means(&self, &img, 0, 0, self.size, self.size),
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
        let Self {
            distance, k_means, ..
        } = self;
        let raw = k_means(&self, img, x, y, w, h);

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

impl KMeansProcImpl {
    const RUNS: u64 = 2;
    const FACTOR_RGB: f32 = 0.0025;
    const FACTOR_LAB: f32 = 10.;
    const MAX_ITER_RGB: usize = 10;
    const MAX_ITER_LAB: usize = 20;

    pub fn new(size: u32, distance: Distance, color_space: ColorSpace) -> Self {
        let converge = match color_space {
            ColorSpace::CIELAB => Self::FACTOR_LAB,
            _ => Self::FACTOR_RGB,
        };

        let max_iter = match color_space {
            ColorSpace::CIELAB => Self::MAX_ITER_LAB,
            _ => Self::MAX_ITER_RGB,
        };

        let k_means: KMeansResult = Box::new(match color_space {
            ColorSpace::CIELAB => Self::k_means::<Lab>,
            _ => Self::k_means::<Srgb>,
        });

        Self {
            size,
            converge,
            max_iter,
            distance,
            k_means,
        }
    }

    // #[inline(always)]
    fn k_means<
        T: Calculate + Sort + Copy + Clone + Clamp + Pixel<f32> + FromColorUnclamped<Srgb>,
    >(
        &self,
        img: &RgbImage,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Vec<RawColor> {
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

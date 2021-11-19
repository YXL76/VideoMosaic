use {
    super::{ColorSpace, Distance, LibItem, Mask, Process},
    crate::utils::{Color, RawColor, HSV, LAB, SRBG},
    image::{self, imageops::FilterType, RgbImage},
    kmeans_colors::{get_kmeans, get_kmeans_hamerly, Kmeans},
    palette::{IntoColor, Pixel, Srgb},
    std::sync::Arc,
};

pub(super) struct KMeansImpl {
    size: u32,
    k: usize,
    converge: f32,
    max_iter: usize,
    filter: FilterType,
    distance: Distance,
    k_means: Box<dyn Fn(usize, usize, f32, &RgbImage, Mask) -> RawColor + Sync + Send>,
}

impl Process for KMeansImpl {
    #[inline(always)]
    fn size(&self) -> u32 {
        self.size
    }

    #[inline(always)]
    fn filter(&self) -> FilterType {
        self.filter
    }

    #[inline(always)]
    fn index_step(&self, img: RgbImage) -> LibItem {
        let Self { k_means, .. } = self;
        (
            vec![k_means(
                self.k,
                self.max_iter,
                self.converge,
                &img,
                (0, 0, self.size, self.size),
            )],
            Arc::new(img),
        )
    }

    #[inline(always)]
    fn fill_step(
        &self,
        img: Arc<RgbImage>,
        mask: Mask,
        lib: Arc<Vec<LibItem>>,
    ) -> (Mask, Arc<RgbImage>) {
        let Self {
            k,
            converge,
            max_iter,
            distance,
            k_means,
            ..
        } = self;
        let raw = k_means(*k, *max_iter, *converge, &img, mask);

        let (_, replace) = lib
            .iter()
            .min_by(|(a, _), (b, _)| {
                distance(&a[0], &raw)
                    .partial_cmp(&distance(&b[0], &raw))
                    .unwrap()
            })
            .unwrap();

        (mask, replace.clone())
    }
}

impl KMeansImpl {
    const FACTOR_RGB: f32 = 0.0025;
    const FACTOR_LAB: f32 = 10.;
    const MAX_ITER_RGB: usize = 10;
    const MAX_ITER_LAB: usize = 20;

    pub(super) fn new(
        size: u32,
        k: usize,
        hamerly: bool,
        filter: FilterType,
        distance: Distance,
        color_space: ColorSpace,
    ) -> Self {
        let converge = match color_space {
            ColorSpace::RGB | ColorSpace::HSV => Self::FACTOR_RGB,
            ColorSpace::CIELAB => Self::FACTOR_LAB,
        };

        let max_iter = match color_space {
            ColorSpace::RGB | ColorSpace::HSV => Self::MAX_ITER_RGB,
            ColorSpace::CIELAB => Self::MAX_ITER_LAB,
        };

        let k_means = Box::new(match hamerly {
            true => match color_space {
                ColorSpace::RGB => k_means_hamerly::<SRBG>,
                ColorSpace::HSV => k_means_hamerly::<HSV>,
                ColorSpace::CIELAB => k_means_hamerly::<LAB>,
            },
            false => match color_space {
                ColorSpace::RGB => k_means_std::<SRBG>,
                ColorSpace::HSV => k_means_std::<HSV>,
                ColorSpace::CIELAB => k_means_std::<LAB>,
            },
        });

        Self {
            size,
            k,
            converge,
            max_iter,
            filter,
            distance,
            k_means,
        }
    }
}

macro_rules! k_means {
    ($f:ident, $name:ident) => {
        // #[inline(always)]
        fn $name<T: Color>(
            k: usize,
            max_iter: usize,
            converge: f32,
            img: &RgbImage,
            (x, y, w, h): Mask,
        ) -> RawColor {
            const RUNS: u64 = 3;

            let mut buf: Vec<T> = Vec::with_capacity((w * h) as usize);
            for j in y..(y + h) {
                for i in x..(x + w) {
                    let color = Srgb::from_raw(&img.get_pixel(i, j).0)
                        .into_format::<f32>()
                        .into_color();
                    buf.push(color)
                }
            }
            let res = (0..RUNS).fold(Kmeans::new(), |res, i| {
                let run_res = $f(k, max_iter, converge, false, &buf, i);
                if run_res.score < res.score {
                    run_res
                } else {
                    res
                }
            });
            let mut res = T::sort_indexed_colors(&res.centroids, &res.indices);
            res.sort_unstable_by(|a, b| (b.percentage).partial_cmp(&a.percentage).unwrap());
            res.into_iter().fold([0f32; 3], |mut ans, i| {
                let raw: [f32; 3] = i.centroid.into_raw();
                ans[0] += raw[0] * i.percentage;
                ans[1] += raw[1] * i.percentage;
                ans[2] += raw[2] * i.percentage;
                ans
            })
        }
    };
}

k_means!(get_kmeans, k_means_std);
k_means!(get_kmeans_hamerly, k_means_hamerly);

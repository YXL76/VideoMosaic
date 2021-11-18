use {
    super::{ColorSpace, Distance, LibItem, Mask, Process},
    crate::utils::{Color, RawColor, HSV, LAB, SRBG},
    image::{self, RgbImage},
    kmeans_colors::{get_kmeans, Kmeans},
    palette::{IntoColor, Pixel, Srgb},
    std::sync::Arc,
};

pub(super) struct KMeansImpl {
    size: u32,
    converge: f32,
    max_iter: usize,
    distance: Distance,
    k_means: Box<dyn Fn(usize, f32, &RgbImage, Mask) -> Vec<RawColor> + Sync + Send>,
}

impl Process for KMeansImpl {
    #[inline(always)]
    fn size(&self) -> u32 {
        self.size
    }

    #[inline(always)]
    fn index_step(&self, img: RgbImage) -> LibItem {
        let Self {
            size,
            converge,
            max_iter,
            k_means,
            ..
        } = self;
        (
            k_means(*max_iter, *converge, &img, (0, 0, *size, *size)),
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
            converge,
            max_iter,
            distance,
            k_means,
            ..
        } = self;
        let raw = k_means(*max_iter, *converge, &img, mask);

        let (_, replace) = lib
            .iter()
            .min_by(|(a, _), (b, _)| {
                distance(&a[0], &raw[0])
                    .partial_cmp(&distance(&b[0], &raw[0]))
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

    pub(super) fn new(size: u32, distance: Distance, color_space: ColorSpace) -> Self {
        let converge = match color_space {
            ColorSpace::RGB | ColorSpace::HSV => Self::FACTOR_RGB,
            ColorSpace::CIELAB => Self::FACTOR_LAB,
        };

        let max_iter = match color_space {
            ColorSpace::RGB | ColorSpace::HSV => Self::MAX_ITER_RGB,
            ColorSpace::CIELAB => Self::MAX_ITER_LAB,
        };

        let k_means = Box::new(match color_space {
            ColorSpace::RGB => k_means::<SRBG>,
            ColorSpace::HSV => k_means::<HSV>,
            ColorSpace::CIELAB => k_means::<LAB>,
        });

        Self {
            size,
            converge,
            max_iter,
            distance,
            k_means,
        }
    }
}

// #[inline(always)]
fn k_means<T: Color>(
    max_iter: usize,
    converge: f32,
    img: &RgbImage,
    (x, y, w, h): Mask,
) -> Vec<RawColor> {
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
        let run_res = get_kmeans(1, max_iter, converge, false, &buf, i);
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

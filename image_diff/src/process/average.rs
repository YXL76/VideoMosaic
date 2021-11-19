use {
    super::{Converter, Distance, LibItem, Mask, Process},
    crate::utils::RawColor,
    image::{self, imageops::FilterType, RgbImage},
    std::sync::Arc,
};

pub(super) struct AverageImpl {
    size: u32,
    filter: FilterType,
    converter: Converter,
    distance: Distance,
}

impl Process for AverageImpl {
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
        (
            vec![self.average(&img, (0, 0, self.size, self.size))],
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
        let Self { distance, .. } = self;
        let raw = self.average(&img, mask);

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

impl AverageImpl {
    #[inline(always)]
    pub(super) fn new(
        size: u32,
        filter: FilterType,
        converter: Converter,
        distance: Distance,
    ) -> Self {
        Self {
            size,
            filter,
            converter,
            distance,
        }
    }

    // #[inline(always)]
    fn average(&self, img: &RgbImage, (x, y, w, h): Mask) -> RawColor {
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

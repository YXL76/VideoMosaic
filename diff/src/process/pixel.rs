use {
    super::{Converter, Distance, LibItem, Mask, Process},
    crate::utils::RawColor,
    image::{self, imageops::FilterType, Pixel, RgbImage},
    std::sync::Arc,
};

pub(super) struct PixelImpl {
    size: u32,
    filter: FilterType,
    converter: Converter,
    distance: Distance,
}

impl Process for PixelImpl {
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
        let Self {
            size, converter, ..
        } = self;
        let mut buf: Vec<RawColor> = Vec::with_capacity((size * size) as usize);
        for j in 0..*size {
            for i in 0..*size {
                buf.push(converter(img.get_pixel(i, j).channels()))
            }
        }
        (buf, img)
    }

    #[inline(always)]
    fn fill_step(
        &self,
        img: Arc<RgbImage>,
        mask: Mask,
        lib: Arc<Vec<Vec<RawColor>>>,
    ) -> (Mask, usize) {
        let (idx, _) = lib
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                self.compare(&img, a, mask)
                    .partial_cmp(&self.compare(&img, b, mask))
                    .unwrap()
            })
            .unwrap();

        (mask, idx)
    }
}

impl PixelImpl {
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
    fn compare(&self, img: &RgbImage, other: &[RawColor], (x, y, w, h): Mask) -> f32 {
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
                    &converter(img.get_pixel(i + x, j + y).channels()),
                    &other[(j * size + i) as usize],
                );
            }
        }
        ans
    }
}

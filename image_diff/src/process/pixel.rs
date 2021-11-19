use {
    super::{Converter, Distance, LibItem, Mask, Process},
    crate::utils::RawColor,
    image::{self, imageops::FilterType, RgbImage},
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
                buf.push(converter(&img.get_pixel(i, j).0))
            }
        }
        (buf, Arc::new(img))
    }

    #[inline(always)]
    fn fill_step(
        &self,
        img: Arc<RgbImage>,
        mask: Mask,
        lib: Arc<Vec<LibItem>>,
    ) -> (Mask, Arc<RgbImage>) {
        let (_, replace) = lib
            .iter()
            .min_by(|(a, _), (b, _)| {
                self.compare(&img, a, mask)
                    .partial_cmp(&self.compare(&img, b, mask))
                    .unwrap()
            })
            .unwrap();

        (mask, replace.clone())
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
    fn compare(&self, img: &RgbImage, other: &Vec<RawColor>, (x, y, w, h): Mask) -> f32 {
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

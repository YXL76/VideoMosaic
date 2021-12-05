use {
    super::{impl_process, Converter, Distance, LibItem, Mask},
    crate::utils::RawColor,
    image::{self, imageops::FilterType, Pixel, RgbImage},
};

pub(super) struct PixelImpl {
    size: u32,
    filter: FilterType,
    converter: Converter,
    distance: Distance,
    lib_color: Box<[RawColor]>,
    lib_image: Box<[RgbImage]>,
    prev: Option<RgbImage>,
    next: Option<RgbImage>,
}

impl_process!(PixelImpl;
    #[inline(always)]
    fn index_step(&self, img: RgbImage) -> LibItem {
        (RawColor::default(), img)
    }

    #[inline(always)]
    fn fill_step(&self, mask: Mask) -> (Mask, usize) {
        let img = self.next.as_ref().unwrap();
        let (idx, _) = self
            .lib_image
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                self.compare(img, a, mask)
                    .partial_cmp(&self.compare(img, b, mask))
                    .unwrap()
            })
            .unwrap();

        (mask, idx)
    }
);

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
            lib_color: Vec::new().into_boxed_slice(),
            lib_image: Vec::new().into_boxed_slice(),
            prev: None,
            next: None,
        }
    }

    // #[inline(always)]
    fn compare(&self, img: &RgbImage, other: &RgbImage, (x, y, w, h): Mask) -> f32 {
        let Self {
            converter,
            distance,
            ..
        } = self;

        let mut ans = 0f32;
        for j in 0..h {
            for i in 0..w {
                ans += distance(
                    &converter(img.get_pixel(i + x, j + y).channels()),
                    &converter(other.get_pixel(i + x, j + y).channels()),
                );
            }
        }
        ans
    }
}

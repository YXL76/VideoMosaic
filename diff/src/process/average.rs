use {
    super::{Converter, Distance, LibItem, Mask, Process},
    crate::utils::RawColor,
    image::{self, imageops::FilterType, Pixel, RgbImage},
};

pub(super) struct AverageImpl {
    size: u32,
    filter: FilterType,
    converter: Converter,
    distance: Distance,
    lib_color: Box<[RawColor]>,
    lib_image: Box<[RgbImage]>,
    prev: Option<RgbImage>,
    next: Option<RgbImage>,
}

impl Process for AverageImpl {
    #[inline(always)]
    fn size(&self) -> u32 {
        self.size
    }

    #[inline(always)]
    fn prev(&self) -> &Option<RgbImage> {
        &self.prev
    }

    #[inline(always)]
    fn next(&self) -> &Option<RgbImage> {
        &self.next
    }

    #[inline(always)]
    fn prev_mut(&mut self) -> &mut Option<RgbImage> {
        &mut self.prev
    }

    #[inline(always)]
    fn next_mut(&mut self) -> &mut Option<RgbImage> {
        &mut self.next
    }

    #[inline(always)]
    fn set_lib(&mut self, lib_color: Vec<RawColor>, lib_image: Vec<RgbImage>) {
        self.lib_color = lib_color.into_boxed_slice();
        self.lib_image = lib_image.into_boxed_slice();
    }

    #[inline(always)]
    fn get_image(&self, idx: usize) -> &RgbImage {
        &self.lib_image[idx]
    }

    #[inline(always)]
    fn filter(&self) -> FilterType {
        self.filter
    }

    #[inline(always)]
    fn index_step(&self, img: RgbImage) -> LibItem {
        (self.average(&img, (0, 0, self.size, self.size)), img)
    }

    #[inline(always)]
    fn fill_step(&self, mask: Mask) -> (Mask, usize) {
        let Self {
            distance,
            lib_color,
            next,
            ..
        } = self;
        let raw = &self.average(next.as_ref().unwrap(), mask);

        let (idx, _) = lib_color
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| distance(a, raw).partial_cmp(&distance(b, raw)).unwrap())
            .unwrap();

        (mask, idx)
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
            lib_color: Vec::new().into_boxed_slice(),
            lib_image: Vec::new().into_boxed_slice(),
            prev: None,
            next: None,
        }
    }

    // #[inline(always)]
    fn average(&self, img: &RgbImage, (x, y, w, h): Mask) -> RawColor {
        let Self { converter, .. } = self;
        let mut ans = [0f32; 3];
        for j in y..(y + h) {
            for i in x..(x + w) {
                let raw = converter(img.get_pixel(i, j).channels());
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

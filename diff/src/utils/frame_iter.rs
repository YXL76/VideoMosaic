use image::RgbImage;

pub trait FrameIter {
    fn new(input: String, output: String) -> (Self, i64, u32, u32)
    where
        Self: Sized;

    fn next(&mut self) -> Option<RgbImage>;

    fn post_next(&mut self, img: &RgbImage);

    fn flush(&mut self) {}
}

pub(crate) struct ImageDump {
    img: Option<RgbImage>,
    output: String,
}

impl FrameIter for ImageDump {
    fn new(input: String, output: String) -> (Self, i64, u32, u32) {
        let img = image::open(input).unwrap().into_rgb8();
        let (width, height) = img.dimensions();
        let img = Some(img);
        (Self { img, output }, 1, width, height)
    }

    fn next(&mut self) -> Option<RgbImage> {
        self.img.take()
    }

    fn post_next(&mut self, img: &RgbImage) {
        let _ = img.save(&self.output);
    }
}

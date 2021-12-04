use {image::RgbImage, std::sync::Arc};

pub trait FrameIter {
    fn new(input: String, output: String) -> (Self, i64, u32, u32)
    where
        Self: Sized;

    fn next(&mut self) -> Option<Arc<RgbImage>>;

    fn post_next(&mut self, img: &RgbImage);

    fn flush(&mut self) {}
}

pub(crate) struct ImageDump {
    img: Option<Arc<RgbImage>>,
    output: String,
}

impl FrameIter for ImageDump {
    fn new(input: String, output: String) -> (Self, i64, u32, u32) {
        let img = image::open(input).unwrap().into_rgb8();
        let (width, height) = img.dimensions();
        (
            Self {
                img: Some(Arc::new(img)),
                output,
            },
            1,
            width,
            height,
        )
    }

    fn next(&mut self) -> Option<Arc<RgbImage>> {
        self.img.take()
    }

    fn post_next(&mut self, img: &RgbImage) {
        let _ = img.save(&self.output);
    }
}

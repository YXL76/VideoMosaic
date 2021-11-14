use {
    super::{Converter, Distance, Process, ProcessResult, ProcessStep, RawColor},
    image::{self, imageops::FilterType, ImageBuffer, RgbImage},
    kmeans_colors::{get_kmeans, Calculate, Kmeans, MapColor, Sort},
    parking_lot::Mutex,
    rayon::prelude::*,
    std::path::PathBuf,
};

pub struct KMeansProcImpl {
    size: u32,
    runs: u64,
    factor: f32,
    max_iter: usize,
    converter: Converter,
    distance: Distance,
}

impl Process for KMeansProcImpl {
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage> {
        self.fill(target, self.index(library)?)
    }
}

impl ProcessStep for KMeansProcImpl {
    type Lib = Vec<(ImgData, Box<RgbImage>)>;

    fn index(&self, libraries: &[PathBuf]) -> ProcessResult<Self::Lib> {
        let Self {
            size, converter, ..
        } = self;

        let vec = Mutex::new(Vec::with_capacity(libraries.len()));
        libraries
            .into_par_iter()
            .for_each(|lib| if let Ok(img) = image::open(lib) {});
        let vec = vec.into_inner();
        if vec.len() == 0 {
            return Err("");
        }
        Ok(vec)
    }

    fn fill(&self, target: &PathBuf, lib: Self::Lib) -> ProcessResult<RgbImage> {
        let img = image::open(target).unwrap().into_rgb8();
        let (width, height) = img.dimensions();
        let imgbuf = Mutex::new(ImageBuffer::new(width, height));

        for y in (0..height).step_by(self.size as usize) {
            (0..width)
                .into_par_iter()
                .step_by(self.size as usize)
                .for_each(|x| {
                    let w = self.size.min(width - x);
                    let h = self.size.min(height - y);
                })
        }

        Ok(imgbuf.into_inner())
    }
}

impl KMeansProcImpl {
    pub fn new(
        size: u32,
        runs: u64,
        factor: f32,
        max_iter: usize,
        converter: Converter,
        distance: Distance,
    ) -> Self {
        Self {
            size,
            runs,
            factor,
            max_iter,
            converter,
            distance,
        }
    }
}

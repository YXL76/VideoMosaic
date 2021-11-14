mod average;
mod kmeans;
mod pixel;

use {
    crate::{CalculationUnit, ColorSpace, DistanceAlgorithm},
    average::AverageProcImpl,
    image::{imageops::FilterType, ImageBuffer, RgbImage},
    kmeans::KMeansProcImpl,
    palette::{encoding, white_point::D65, ColorDifference, Hsv, IntoColor, Lab, Pixel, Srgb},
    parking_lot::Mutex,
    pixel::PixelProcImpl,
    rayon::prelude::*,
    std::path::PathBuf,
};

type RawColor = [f32; 3];
type ProcessResult<T> = Result<T, &'static str>;
type Converter = Box<dyn Fn(&[u8; 3]) -> RawColor + Sync + Send>;
type Distance = Box<dyn Fn(&RawColor, &RawColor) -> f32 + Sync + Send>;
type KMeansResult =
    Box<dyn Fn(&KMeansProcImpl, &RgbImage, u32, u32, u32, u32) -> Vec<RawColor> + Sync + Send>;

trait Process {
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage>;
}

trait ProcessStep {
    type Item: Sync + Send;

    fn size(&self) -> u32;

    #[inline(always)]
    fn index(&self, libraries: &[PathBuf]) -> ProcessResult<Vec<Self::Item>>
    where
        Self: Sync + Send,
    {
        let vec = Mutex::new(Vec::with_capacity(libraries.len()));
        libraries.into_par_iter().for_each(|lib| {
            if let Ok(img) = image::open(lib) {
                let img = img
                    .resize_to_fill(self.size(), self.size(), FilterType::Nearest)
                    .into_rgb8();
                vec.lock().push(self.index_step(img));
            }
        });
        let vec = vec.into_inner();
        if vec.len() == 0 {
            return Err("");
        }
        Ok(vec)
    }

    #[inline(always)]
    fn fill(&self, target: &PathBuf, lib: Vec<Self::Item>) -> ProcessResult<RgbImage>
    where
        Self: Sync + Send,
    {
        let img = image::open(target).unwrap().into_rgb8();
        let (width, height) = img.dimensions();
        let imgbuf: Mutex<RgbImage> = Mutex::new(ImageBuffer::new(width, height));

        for y in (0..height).step_by(self.size() as usize) {
            (0..width)
                .into_par_iter()
                .step_by(self.size() as usize)
                .for_each(|x| {
                    let w = self.size().min(width - x);
                    let h = self.size().min(height - y);
                    self.fill_step(&img, x, y, w, h, &lib, &imgbuf);
                })
        }

        Ok(imgbuf.into_inner())
    }

    fn index_step(&self, img: RgbImage) -> Self::Item;

    fn fill_step(
        &self,
        img: &RgbImage,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        lib: &Vec<Self::Item>,
        buf: &Mutex<RgbImage>,
    );
}

pub struct ProcessWrapper(Box<dyn Process>);

impl ProcessWrapper {
    pub fn new(
        size: u32,
        calc_unit: CalculationUnit,
        color_space: ColorSpace,
        dist_algo: DistanceAlgorithm,
    ) -> Self {
        let converter = Box::new(match color_space {
            ColorSpace::RGB => |rgb: &[u8; 3]| Srgb::from_raw(rgb).into_format::<f32>().into_raw(),
            ColorSpace::HSV => |rgb: &[u8; 3]| {
                let hsv: Hsv<_, f32> = Srgb::from_raw(rgb).into_format::<f32>().into_color();
                hsv.into_raw()
            },
            ColorSpace::CIELAB => |rgb: &[u8; 3]| {
                let lab: Lab<_, f32> = Srgb::from_raw(rgb).into_format::<f32>().into_color();
                lab.into_raw()
            },
        });

        let distance = Box::new(match dist_algo {
            DistanceAlgorithm::Euclidean => |a: &RawColor, b: &RawColor| {
                (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
            },
            DistanceAlgorithm::CIEDE2000 => match color_space {
                ColorSpace::RGB => |a: &RawColor, b: &RawColor| {
                    let a: Lab<D65, f32> = (*Srgb::from_raw(a)).into_color();
                    let b: Lab<D65, f32> = (*Srgb::from_raw(b)).into_color();
                    a.get_color_difference(&b)
                },
                ColorSpace::HSV => |a: &RawColor, b: &RawColor| {
                    let a: Lab<D65, f32> = (*Hsv::<encoding::Srgb, f32>::from_raw(a)).into_color();
                    let b: Lab<D65, f32> = (*Hsv::<encoding::Srgb, f32>::from_raw(b)).into_color();
                    a.get_color_difference(&b)
                },
                ColorSpace::CIELAB => |a: &RawColor, b: &RawColor| {
                    let a: &Lab<D65, f32> = Lab::from_raw(a);
                    let b: &Lab<D65, f32> = Lab::from_raw(b);
                    a.get_color_difference(b)
                },
            },
        });

        Self(match calc_unit {
            CalculationUnit::Average => Box::new(AverageProcImpl::new(size, converter, distance)),
            CalculationUnit::Pixel => Box::new(PixelProcImpl::new(size, converter, distance)),
            CalculationUnit::KMeans => Box::new(KMeansProcImpl::new(size, distance, color_space)),
        })
    }

    pub fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage> {
        self.0.run(target, library)
    }
}

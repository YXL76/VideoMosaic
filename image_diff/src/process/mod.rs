mod average;
mod kmeans;
mod pixel;

use {
    crate::{CalculationUnit, ColorSpace, DistanceAlgorithm},
    average::AverageProc,
    image::{imageops::FilterType, ImageBuffer, RgbImage},
    kmeans::KMeansProc,
    kmeans_colors::{Calculate, Sort},
    palette::{
        convert::FromColorUnclamped, encoding, white_point::D65, Clamp, ColorDifference, Hsv,
        IntoColor, Lab, Pixel, Srgb,
    },
    parking_lot::Mutex,
    pixel::PixelProc,
    rayon::prelude::*,
    std::path::PathBuf,
};

type SRBG = Srgb<f32>;
type HSV = Hsv<encoding::Srgb, f32>;
type LAB = Lab<D65, f32>;

type RawColor = [f32; 3];
type ProcessResult<T> = Result<T, &'static str>;
type Distance = Box<dyn Fn(&RawColor, &RawColor) -> f32 + Sync + Send>;

pub trait Color = Copy
    + Clone
    + Calculate
    + Sort
    + Clamp
    + Pixel<f32>
    + FromColorUnclamped<palette::rgb::Rgb>
    + Sync
    + Send;

trait Process {
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage>;
}

trait ProcessStep<T: Color> {
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
        if vec.is_empty() {
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

    #[inline(always)]
    fn converter(rgb: &[u8; 3]) -> RawColor {
        let color: T = Srgb::from_raw(rgb).into_format::<f32>().into_color();
        color.into_raw()
    }
}

pub struct ProcessWrapper(Box<dyn Process>);

impl ProcessWrapper {
    pub fn new(
        size: u32,
        calc_unit: CalculationUnit,
        color_space: ColorSpace,
        dist_algo: DistanceAlgorithm,
    ) -> Self {
        let distance = Box::new(match dist_algo {
            DistanceAlgorithm::Euclidean => match color_space {
                ColorSpace::HSV => |a: &RawColor, b: &RawColor| {
                    let a = HSV::from_raw(a);
                    let b = HSV::from_raw(b);
                    (a.hue.to_positive_degrees() - b.hue.to_positive_degrees()).powi(2)
                        + ((a.saturation - b.saturation) * 360.).powi(2)
                        + ((a.value - b.value) * 360.).powi(2)
                },
                _ => |a: &RawColor, b: &RawColor| {
                    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
                },
            },
            DistanceAlgorithm::CIEDE2000 => match color_space {
                ColorSpace::RGB => ciede2000::<SRBG>,
                ColorSpace::HSV => ciede2000::<HSV>,
                ColorSpace::CIELAB => ciede2000::<Lab>,
            },
        });

        Self(match calc_unit {
            CalculationUnit::Average => match color_space {
                ColorSpace::RGB => Box::new(AverageProc::<SRBG>::new(size, distance)),
                ColorSpace::HSV => Box::new(AverageProc::<HSV>::new(size, distance)),
                ColorSpace::CIELAB => Box::new(AverageProc::<Lab>::new(size, distance)),
            },
            CalculationUnit::Pixel => match color_space {
                ColorSpace::RGB => Box::new(PixelProc::<SRBG>::new(size, distance)),
                ColorSpace::HSV => Box::new(PixelProc::<HSV>::new(size, distance)),
                ColorSpace::CIELAB => Box::new(PixelProc::<Lab>::new(size, distance)),
            },
            CalculationUnit::KMeans => match color_space {
                ColorSpace::RGB => Box::new(KMeansProc::<SRBG>::new(size, distance, color_space)),
                ColorSpace::HSV => Box::new(KMeansProc::<HSV>::new(size, distance, color_space)),
                ColorSpace::CIELAB => Box::new(KMeansProc::<Lab>::new(size, distance, color_space)),
            },
        })
    }

    pub fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage> {
        self.0.run(target, library)
    }
}

fn ciede2000<T: Copy + Pixel<f32> + IntoColor<LAB>>(a: &RawColor, b: &RawColor) -> f32 {
    let a: LAB = (*T::from_raw(a)).into_color();
    let b: LAB = (*T::from_raw(b)).into_color();
    a.get_color_difference(&b)
}

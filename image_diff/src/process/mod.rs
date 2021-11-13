mod average;
mod pixel;

use {
    crate::{CalculationUnit, ColorSpace, DistanceAlgorithm},
    average::AverageProcImpl,
    image::RgbImage,
    palette::{encoding, white_point::D65, ColorDifference, Hsv, IntoColor, Lab, Pixel, Srgb},
    pixel::PixelProcImpl,
    std::path::PathBuf,
};

type ProcessResult<T> = Result<T, &'static str>;
type Converter = Box<dyn Fn(&[u8; 3]) -> [f64; 3] + Sync + Send>;
type Distance = Box<dyn Fn(&[f64; 3], &[f64; 3]) -> f64 + Sync + Send>;

trait Process {
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage>;
}

trait ProcessStep {
    type Lib;

    fn index(&self, libraries: &[PathBuf]) -> ProcessResult<Self::Lib>;

    fn fill(&self, target: &PathBuf, lib: Self::Lib) -> ProcessResult<RgbImage>;
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
            ColorSpace::RGB => |rgb: &[u8; 3]| Srgb::from_raw(rgb).into_format::<f64>().into_raw(),
            ColorSpace::HSV => |rgb: &[u8; 3]| {
                let hsv: Hsv<_, f64> = Srgb::from_raw(rgb).into_format::<f64>().into_color();
                hsv.into_raw()
            },
            ColorSpace::CIELAB => |rgb: &[u8; 3]| {
                let lab: Lab<_, f64> = Srgb::from_raw(rgb).into_format::<f64>().into_color();
                lab.into_raw()
            },
        });

        let distance = Box::new(match dist_algo {
            DistanceAlgorithm::Euclidean => |a: &[f64; 3], b: &[f64; 3]| {
                (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
            },
            DistanceAlgorithm::CIEDE2000 => match color_space {
                ColorSpace::RGB => |a: &[f64; 3], b: &[f64; 3]| {
                    let a: Lab<D65, f64> = (*Srgb::from_raw(a)).into_color();
                    let b: Lab<D65, f64> = (*Srgb::from_raw(b)).into_color();
                    a.get_color_difference(&b)
                },
                ColorSpace::HSV => |a: &[f64; 3], b: &[f64; 3]| {
                    let a: Lab<D65, f64> = (*Hsv::<encoding::Srgb, f64>::from_raw(a)).into_color();
                    let b: Lab<D65, f64> = (*Hsv::<encoding::Srgb, f64>::from_raw(b)).into_color();
                    a.get_color_difference(&b)
                },
                ColorSpace::CIELAB => |a: &[f64; 3], b: &[f64; 3]| {
                    let a: &Lab<D65, f64> = Lab::from_raw(a);
                    let b: &Lab<D65, f64> = Lab::from_raw(b);
                    a.get_color_difference(b)
                },
            },
        });

        Self(match calc_unit {
            CalculationUnit::Average => Box::new(AverageProcImpl::new(size, converter, distance)),
            CalculationUnit::Pixel => Box::new(PixelProcImpl::new(size, converter, distance)),
            _ => Box::new(AverageProcImpl::new(size, converter, distance)),
        })
    }

    pub fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage> {
        self.0.run(target, library)
    }
}

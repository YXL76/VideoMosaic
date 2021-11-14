mod average;
mod kmeans;
mod pixel;

use {
    crate::{CalculationUnit, ColorSpace, DistanceAlgorithm},
    average::AverageProcImpl,
    image::RgbImage,
    kmeans::KMeansProcImpl,
    palette::{encoding, white_point::D65, ColorDifference, Hsv, IntoColor, Lab, Pixel, Srgb},
    pixel::PixelProcImpl,
    std::path::PathBuf,
};

type RawColor = [f32; 3];
type ProcessResult<T> = Result<T, &'static str>;
type Converter = Box<dyn Fn(&[u8; 3]) -> RawColor + Sync + Send>;
type Distance = Box<dyn Fn(&RawColor, &RawColor) -> f32 + Sync + Send>;

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
            CalculationUnit::KMeans => {
                const RUNS: u64 = 2;
                const FACTOR_RGB: f32 = 0.0025;
                const FACTOR_LAB: f32 = 10.;
                const MAX_ITER_RGB: usize = 10;
                const MAX_ITER_LAB: usize = 20;

                let factor = match color_space {
                    ColorSpace::CIELAB => FACTOR_LAB,
                    _ => FACTOR_RGB,
                };
                let max_iter = match color_space {
                    ColorSpace::CIELAB => MAX_ITER_LAB,
                    _ => MAX_ITER_RGB,
                };

                Box::new(KMeansProcImpl::new(
                    size, RUNS, factor, max_iter, converter, distance,
                ))
            }
        })
    }

    pub fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<RgbImage> {
        self.0.run(target, library)
    }
}

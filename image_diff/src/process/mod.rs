mod average;

use {
    crate::{CalculationUnit, ColorSpace, DistanceAlgorithm},
    average::AverageProcImpl,
    std::path::PathBuf,
};

type ProcessResult<T> = Result<T, &'static str>;

trait Process {
    fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<&str>;
}

pub struct ProcessFactory(Box<dyn Process>);

impl ProcessFactory {
    pub fn new(config: ProcessConfig) -> Self {
        Self(Box::new(AverageProcImpl::new(config)))
    }

    pub fn run(&self, target: &PathBuf, library: &[PathBuf]) -> ProcessResult<&str> {
        self.0.run(target, library)
    }
}

pub struct ProcessConfig {
    size: u32,
    calc_unit: CalculationUnit,
    color_space: ColorSpace,
    dist_algo: DistanceAlgorithm,
}

impl ProcessConfig {
    pub fn new(
        size: u32,
        calc_unit: CalculationUnit,
        color_space: ColorSpace,
        dist_algo: DistanceAlgorithm,
    ) -> Self {
        Self {
            size,
            calc_unit,
            color_space,
            dist_algo,
        }
    }
}

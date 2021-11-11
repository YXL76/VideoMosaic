#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CalculationUnit {
    Average,
    Pixel,
    KMeans,
}

impl Default for CalculationUnit {
    fn default() -> Self {
        Self::KMeans
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ColorSpace {
    RGB,
    HSV,
    CIEXYZ,
}

impl Default for ColorSpace {
    fn default() -> Self {
        Self::CIEXYZ
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DistanceAlgorithm {
    Euclidean,
    CIEDE2000,
}

impl Default for DistanceAlgorithm {
    fn default() -> Self {
        Self::CIEDE2000
    }
}

mod frame_iter;
mod transcoder;
mod variance;

use {
    kmeans_colors::{Calculate, Hamerly, Sort},
    palette::{
        convert::FromColorUnclamped, encoding, white_point::D65, Clamp, ColorDifference, Hsv,
        IntoColor, Lab, Pixel, Srgb,
    },
    std::cmp::{Ord, Ordering, PartialEq},
};

pub(crate) use {
    frame_iter::FrameIter, frame_iter::ImageDump, transcoder::Transcode, variance::Variance,
};

pub const IMAGE_FILTER: [&str; 3] = ["png", "jpg", "jpeg"];
pub const VIDEO_FILTER: [&str; 1] = ["mp4"];

pub(crate) type MySrgb = Srgb<f32>;
pub(crate) type MyHsv = Hsv<encoding::Srgb, f32>;
pub(crate) type MyLab = Lab<D65, f32>;
pub(crate) type RawColor = [f32; 3];

pub trait Color:
    Copy
    + Clone
    + Hamerly
    + Calculate
    + Sort
    + Clamp
    + Pixel<f32>
    + FromColorUnclamped<palette::rgb::Rgb>
    + Sync
    + Send
{
}

impl Color for MySrgb {}
impl Color for MyHsv {}
impl Color for MyLab {}

#[inline(always)]
pub(crate) fn converter<T: Color>(rgb: &[u8; 3]) -> RawColor {
    let color: T = Srgb::from_raw(rgb).into_format::<f32>().into_color();
    color.into_raw()
}

pub(crate) fn ciede2000<T: Copy + Pixel<f32> + IntoColor<MyLab>>(
    a: &RawColor,
    b: &RawColor,
) -> f32 {
    let a: MyLab = (*T::from_raw(a)).into_color();
    let b: MyLab = (*T::from_raw(b)).into_color();
    a.get_color_difference(&b)
}

pub(crate) struct F32Wrapper(pub f32);

impl PartialEq for F32Wrapper {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        if self.0.is_nan() {
            other.0.is_nan()
        } else {
            self.0 == other.0
        }
    }
}

impl Eq for F32Wrapper {}

impl PartialOrd for F32Wrapper {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for F32Wrapper {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        let lhs = &self.0;
        let rhs = &other.0;
        match lhs.partial_cmp(rhs) {
            Some(ordering) => ordering,
            None => {
                if lhs.is_nan() {
                    if rhs.is_nan() {
                        Ordering::Equal
                    } else {
                        Ordering::Greater
                    }
                } else {
                    Ordering::Less
                }
            }
        }
    }
}

use {
    kmeans_colors::{Calculate, Sort},
    palette::{
        convert::FromColorUnclamped, encoding, white_point::D65, Clamp, ColorDifference, Hsv,
        IntoColor, Lab, Pixel, Srgb,
    },
};

pub(crate) type SRBG = Srgb<f32>;
pub(crate) type HSV = Hsv<encoding::Srgb, f32>;
pub(crate) type LAB = Lab<D65, f32>;
pub(crate) type RawColor = [f32; 3];

pub trait Color = Copy
    + Clone
    + Calculate
    + Sort
    + Clamp
    + Pixel<f32>
    + FromColorUnclamped<palette::rgb::Rgb>
    + Sync
    + Send;

pub(crate) fn ciede2000<T: Copy + Pixel<f32> + IntoColor<LAB>>(a: &RawColor, b: &RawColor) -> f32 {
    let a: LAB = (*T::from_raw(a)).into_color();
    let b: LAB = (*T::from_raw(b)).into_color();
    a.get_color_difference(&b)
}

#![feature(int_log)]
#![feature(map_first_last)]

extern crate ffmpeg_next as ffmpeg;

mod process;
mod utils;

use {
    ffmpeg::{
        format::{input, Pixel},
        media,
        util::frame::video::Video,
    },
    std::path::Path,
};

pub use {ffmpeg::Error, process::*, utils::*};

pub fn init() {
    ffmpeg::init().unwrap();
    ffmpeg::log::set_level(ffmpeg::log::Level::Quiet);
}

pub fn first_frame(filename: &Path) -> Result<(u32, u32, Vec<u8>), ffmpeg::Error> {
    let mut ictx = input(&filename)?;
    let input = ictx
        .streams()
        .best(media::Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;

    let stream_idx = input.index();
    let mut decoder = input.codec().decoder().video()?;
    let mut converter = decoder.converter(Pixel::BGRA)?;

    for (stream, packet) in ictx.packets() {
        if stream.index() == stream_idx {
            decoder.send_packet(&packet)?;
            let mut decoded = Video::empty();
            if decoder.receive_frame(&mut decoded).is_ok() {
                let mut rgb_frame = Video::empty();
                converter.run(&decoded, &mut rgb_frame)?;
                return Ok((
                    decoder.width(),
                    decoder.height(),
                    rgb_frame.data(0).to_owned(),
                ));
            }
        }
    }

    Err(ffmpeg::Error::InvalidData)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CalculationUnit {
    Average,
    Pixel,
    KMeans,
}

impl Default for CalculationUnit {
    fn default() -> Self {
        Self::Average
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ColorSpace {
    RGB,
    HSV,
    CIELAB,
}

impl Default for ColorSpace {
    fn default() -> Self {
        Self::RGB
    }
}

impl From<ColorSpace> for String {
    fn from(cs: ColorSpace) -> Self {
        Self::from(match cs {
            ColorSpace::RGB => "RGB",
            ColorSpace::HSV => "HSV",
            ColorSpace::CIELAB => "CIE L*a*b*",
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DistanceAlgorithm {
    Euclidean,
    CIEDE2000,
}

impl Default for DistanceAlgorithm {
    fn default() -> Self {
        Self::Euclidean
    }
}

impl From<DistanceAlgorithm> for String {
    fn from(da: DistanceAlgorithm) -> Self {
        Self::from(match da {
            DistanceAlgorithm::Euclidean => "Euclidean",
            DistanceAlgorithm::CIEDE2000 => "CIEDE2000",
        })
    }
}

pub fn str2cu(cu: &str) -> Result<CalculationUnit, String> {
    match cu {
        "average" => Ok(CalculationUnit::Average),
        "pixel" => Ok(CalculationUnit::Pixel),
        "k_means" => Ok(CalculationUnit::KMeans),
        _ => Err("incorrect calculation unit".into()),
    }
}

pub fn str2cs(cs: &str) -> Result<ColorSpace, String> {
    match cs {
        "rgb" => Ok(ColorSpace::RGB),
        "hsv" => Ok(ColorSpace::HSV),
        "cielab" => Ok(ColorSpace::CIELAB),
        _ => Err("incorrect color space".into()),
    }
}

pub fn str2da(da: &str) -> Result<DistanceAlgorithm, String> {
    match da {
        "euclidean" => Ok(DistanceAlgorithm::Euclidean),
        "ciede2000" => Ok(DistanceAlgorithm::CIEDE2000),
        _ => Err("incorrect distance algorithm".into()),
    }
}

pub fn str2filter(filter: &str) -> Result<Filter, String> {
    match filter {
        "nearest" => Ok(Filter::Nearest),
        "triangle" => Ok(Filter::Triangle),
        "catmullRom" => Ok(Filter::CatmullRom),
        "gaussian" => Ok(Filter::Gaussian),
        "lanczos3" => Ok(Filter::Lanczos3),
        _ => Err("incorrect filter".into()),
    }
}

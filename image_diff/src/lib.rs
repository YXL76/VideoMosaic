#![feature(trait_alias)]

extern crate ffmpeg_next as ffmpeg;

mod process;

use {
    ffmpeg::{
        format::{input, Pixel},
        media,
        util::frame::video::Video,
    },
    std::path::PathBuf,
};

pub use {ffmpeg::Error, process::*};

pub fn init() -> Result<(), ffmpeg::Error> {
    ffmpeg::init()
}

pub fn first_frame(filename: &PathBuf) -> Result<(u32, u32, Vec<u8>), ffmpeg::Error> {
    let mut ictx = input(filename)?;
    let input = ictx
        .streams()
        .best(media::Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;

    let stream_idx = input.index();
    let mut decoder = input.codec().decoder().video()?;
    let mut scaler = decoder.converter(Pixel::BGRA)?;

    for (stream, packet) in ictx.packets() {
        if stream.index() == stream_idx {
            decoder.send_packet(&packet)?;
            let mut decoded = Video::empty();
            if decoder.receive_frame(&mut decoded).is_ok() {
                let mut rgb_frame = Video::empty();
                scaler.run(&decoded, &mut rgb_frame)?;
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
        Self::KMeans
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
        Self::CIELAB
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

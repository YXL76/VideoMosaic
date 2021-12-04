use {
    super::FrameIter,
    ffmpeg::{
        codec, decoder, encoder, format, frame, media, picture, software, Dictionary, Packet,
        Rational,
    },
    image::RgbImage,
    std::{collections::HashMap, sync::Arc},
};

pub(crate) struct Transcode {
    ictx: format::context::Input,
    octx: format::context::Output,
    stream_mapping: Vec<isize>,
    ist_time_bases: Vec<Rational>,
    ost_time_bases: Vec<Rational>,
    transcoders: HashMap<usize, Transcoder>,

    last: Option<(usize, Rational, Option<i64>, frame::Video)>,
}

unsafe impl Sync for Transcode {}

impl FrameIter for Transcode {
    fn new(input: String, output: String) -> (Self, i64, u32, u32) {
        let ictx = format::input(&input).unwrap();
        let mut octx = format::output(&output).unwrap();
        format::context::input::dump(&ictx, 0, Some(&input));

        let mut stream_mapping: Vec<isize> = vec![0; ictx.nb_streams() as _];
        let mut ist_time_bases = vec![Rational(0, 0); ictx.nb_streams() as _];
        let mut ost_time_bases = vec![Rational(0, 0); ictx.nb_streams() as _];
        let mut transcoders = HashMap::new();
        let mut ost_index = 0;

        let mut cnt = 0;
        for (ist_index, ist) in ictx.streams().enumerate() {
            cnt = cnt.max(ist.frames());
            let ist_medium = ist.codec().medium();
            if ist_medium != media::Type::Audio
                && ist_medium != media::Type::Video
                && ist_medium != media::Type::Subtitle
            {
                stream_mapping[ist_index] = -1;
                continue;
            }
            stream_mapping[ist_index] = ost_index;
            ist_time_bases[ist_index] = ist.time_base();
            if ist_medium == media::Type::Video {
                transcoders.insert(
                    ist_index,
                    Transcoder::new(&ist, &mut octx, ost_index as _).unwrap(),
                );
            } else {
                let mut ost = octx.add_stream(encoder::find(codec::Id::None)).unwrap();
                ost.set_parameters(ist.parameters());
                unsafe {
                    (*ost.parameters().as_mut_ptr()).codec_tag = 0;
                }
            }
            ost_index += 1;
        }

        octx.set_metadata(ictx.metadata().to_owned());
        format::context::output::dump(&octx, 0, Some(&output));
        octx.write_header().unwrap();

        for (ost_index, _) in octx.streams().enumerate() {
            ost_time_bases[ost_index] = octx.stream(ost_index as _).unwrap().time_base();
        }

        let rnd_frame = transcoders.values().next().unwrap();
        let width = rnd_frame.width();
        let height = rnd_frame.height();
        (
            Self {
                ictx,
                octx,
                stream_mapping,
                ist_time_bases,
                ost_time_bases,
                transcoders,

                last: None,
            },
            cnt,
            width,
            height,
        )
    }

    fn next(&mut self) -> Option<Arc<RgbImage>> {
        let Self {
            ictx,
            octx,
            stream_mapping,
            ist_time_bases,
            ost_time_bases,
            transcoders,
            last,
        } = self;

        if let Some((ist_index, _, old_timestamp, old_frame)) = last {
            if let Some(transcoder) = transcoders.get_mut(ist_index) {
                // transcoder.receive_and_process_decoded_frames(octx, ost_time_base);
                if let Some((timestamp, frame)) = transcoder.receive_decoded_frames() {
                    let width = transcoder.width();
                    let height = transcoder.height();
                    let buf = frame.data(0).to_owned();
                    *old_timestamp = timestamp;
                    *old_frame = frame;
                    let img = RgbImage::from_raw(width, height, buf).unwrap();
                    return Some(Arc::new(img));
                }
            }
        }
        *last = None;

        for (stream, mut packet) in ictx.packets() {
            let ist_index = stream.index();
            let ost_index = stream_mapping[ist_index];
            if ost_index < 0 {
                continue;
            }
            let ost_time_base = ost_time_bases[ost_index as usize];
            match transcoders.get_mut(&ist_index) {
                Some(transcoder) => {
                    packet.rescale_ts(stream.time_base(), transcoder.decoder.time_base());
                    transcoder.send_packet_to_decoder(&packet);
                    // transcoder.receive_and_process_decoded_frames(octx, ost_time_base);
                    if let Some((timestamp, frame)) = transcoder.receive_decoded_frames() {
                        let width = transcoder.width();
                        let height = transcoder.height();
                        let buf = frame.data(0).to_owned();
                        *last = Some((ist_index, ost_time_base, timestamp, frame));
                        let img = RgbImage::from_raw(width, height, buf).unwrap();
                        return Some(Arc::new(img));
                    }
                }
                None => {
                    packet.rescale_ts(ist_time_bases[ist_index], ost_time_base);
                    packet.set_position(-1);
                    packet.set_stream(ost_index as _);
                    packet.write_interleaved(octx).unwrap();
                }
            }
        }

        None
    }

    fn post_next(&mut self, img: &RgbImage) {
        let Self {
            octx,
            transcoders,
            last,
            ..
        } = self;
        if let Some((ist_index, ost_time_base, timestamp, frame)) = last {
            if let Some(transcoder) = transcoders.get_mut(ist_index) {
                // transcoder.receive_and_process_decoded_frames(octx, ost_time_base);
                frame.data_mut(0).copy_from_slice(img.as_raw());
                transcoder.process_decoded_frames(*timestamp, frame, octx, *ost_time_base);
            }
        }
    }

    fn flush(&mut self) {
        let Self {
            octx,
            ost_time_bases,
            transcoders,
            ..
        } = self;

        for (ost_index, transcoder) in transcoders.iter_mut() {
            let ost_time_base = ost_time_bases[*ost_index];
            transcoder.send_eof_to_decoder();
            transcoder.receive_and_process_decoded_frames(octx, ost_time_base);
            transcoder.send_eof_to_encoder();
            transcoder.receive_and_process_encoded_packets(octx, ost_time_base);
        }

        octx.write_trailer().unwrap();

        // to be dropped automatic
        // self.stream_mapping.clear();
        // self.stream_mapping.shrink_to_fit();
        // self.ist_time_bases.clear();
        // self.ist_time_bases.shrink_to_fit();
        // self.ost_time_bases.clear();
        // self.ost_time_bases.shrink_to_fit();
        // self.transcoders.clear();
        // self.transcoders.shrink_to_fit();
    }
}

struct Transcoder {
    ost_index: usize,
    pub(super) decoder: decoder::Video,
    encoder: encoder::video::Video,
}

impl Transcoder {
    fn new(
        ist: &format::stream::Stream,
        octx: &mut format::context::Output,
        ost_index: usize,
    ) -> Result<Self, ffmpeg::Error> {
        let global_header = octx.format().flags().contains(format::Flags::GLOBAL_HEADER);
        let decoder = ist.codec().decoder().video()?;
        let mut ost = octx.add_stream(encoder::find(codec::Id::H264))?;
        let mut encoder = ost.codec().encoder().video()?;
        encoder.set_height(decoder.height());
        encoder.set_width(decoder.width());
        encoder.set_aspect_ratio(decoder.aspect_ratio());
        encoder.set_format(decoder.format());
        encoder.set_frame_rate(decoder.frame_rate());
        encoder.set_time_base(decoder.frame_rate().unwrap().invert());
        if global_header {
            encoder.set_flags(codec::Flags::GLOBAL_HEADER);
        }
        let mut x264_opts = Dictionary::new();
        x264_opts.set("preset", "medium");
        encoder.open_with(x264_opts)?;
        encoder = ost.codec().encoder().video()?;
        ost.set_parameters(encoder);
        let encoder = ost.codec().encoder().video()?;
        Ok(Self {
            ost_index,
            decoder,
            encoder,
        })
    }

    #[inline(always)]
    fn width(&self) -> u32 {
        self.decoder.width()
    }

    #[inline(always)]
    fn height(&self) -> u32 {
        self.decoder.height()
    }

    #[inline(always)]
    fn send_packet_to_decoder(&mut self, packet: &Packet) {
        self.decoder.send_packet(packet).unwrap();
    }

    #[inline(always)]
    fn send_eof_to_decoder(&mut self) {
        self.decoder.send_eof().unwrap();
    }

    fn receive_decoded_frames(&mut self) -> Option<(Option<i64>, frame::Video)> {
        let mut decoded = frame::Video::empty();
        if self.decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgb_frame = frame::Video::empty();
            let timestamp = decoded.timestamp();
            self.decoder
                .converter(format::Pixel::RGB24)
                .unwrap()
                .run(&decoded, &mut rgb_frame)
                .unwrap();
            return Some((timestamp, rgb_frame));
        }
        None
    }

    fn process_decoded_frames(
        &mut self,
        timestamp: Option<i64>,
        frame: &frame::Video,
        octx: &mut format::context::Output,
        ost_time_base: Rational,
    ) {
        let mut decoded = frame::Video::empty();
        software::scaling::Context::get(
            format::Pixel::RGB24,
            self.decoder.width(),
            self.decoder.height(),
            self.decoder.format(),
            self.decoder.width(),
            self.decoder.height(),
            software::scaling::Flags::FAST_BILINEAR,
        )
        .unwrap()
        .run(frame, &mut decoded)
        .unwrap();
        decoded.set_pts(timestamp);
        decoded.set_kind(picture::Type::None);
        self.send_frame_to_encoder(&decoded);
        self.receive_and_process_encoded_packets(octx, ost_time_base);
    }

    fn receive_and_process_decoded_frames(
        &mut self,
        octx: &mut format::context::Output,
        ost_time_base: Rational,
    ) {
        let mut frame = frame::Video::empty();
        while self.decoder.receive_frame(&mut frame).is_ok() {
            let timestamp = frame.timestamp();
            frame.set_pts(timestamp);
            frame.set_kind(picture::Type::None);
            self.send_frame_to_encoder(&frame);
            self.receive_and_process_encoded_packets(octx, ost_time_base);
        }
    }

    #[inline(always)]
    fn send_frame_to_encoder(&mut self, frame: &frame::Video) {
        self.encoder.send_frame(frame).unwrap();
    }

    #[inline(always)]
    fn send_eof_to_encoder(&mut self) {
        self.encoder.send_eof().unwrap();
    }

    fn receive_and_process_encoded_packets(
        &mut self,
        octx: &mut format::context::Output,
        ost_time_base: Rational,
    ) {
        let mut encoded = Packet::empty();
        while self.encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(self.ost_index);
            encoded.rescale_ts(self.decoder.time_base(), ost_time_base);
            encoded.write_interleaved(octx).unwrap();
        }
    }
}

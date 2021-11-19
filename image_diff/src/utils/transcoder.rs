use {
    ffmpeg::{
        codec, decoder, encoder, format, frame, media, picture, Dictionary, Packet, Rational,
    },
    std::collections::HashMap,
};

pub(crate) struct Transcode {
    ictx: format::context::Input,
    octx: format::context::Output,
    stream_mapping: Vec<isize>,
    ist_time_bases: Vec<Rational>,
    ost_time_bases: Vec<Rational>,
    transcoders: HashMap<usize, Transcoder>,
}

impl Transcode {
    fn new(input: String, output: String) -> Self {
        let ictx = format::input(&input).unwrap();
        let mut octx = format::output(&output).unwrap();
        format::context::input::dump(&ictx, 0, Some(&input));

        let mut stream_mapping: Vec<isize> = vec![0; ictx.nb_streams() as _];
        let mut ist_time_bases = vec![Rational(0, 0); ictx.nb_streams() as _];
        let mut ost_time_bases = vec![Rational(0, 0); ictx.nb_streams() as _];
        let mut transcoders = HashMap::new();
        let mut ost_index = 0;

        for (ist_index, ist) in ictx.streams().enumerate() {
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

        Self {
            ictx,
            octx,
            stream_mapping,
            ist_time_bases,
            ost_time_bases,
            transcoders,
        }
    }

    fn next(&mut self) {
        let Self {
            ictx,
            octx,
            stream_mapping,
            ist_time_bases,
            ost_time_bases,
            transcoders,
        } = self;

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
                    transcoder.receive_and_process_decoded_frames(octx, ost_time_base);
                }
                None => {
                    packet.rescale_ts(ist_time_bases[ist_index], ost_time_base);
                    packet.set_position(-1);
                    packet.set_stream(ost_index as _);
                    packet.write_interleaved(octx).unwrap();
                }
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
    }
}

pub(crate) struct Transcoder {
    ost_index: usize,
    decoder: decoder::Video,
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
        Ok(Self {
            ost_index,
            decoder,
            encoder: ost.codec().encoder().video()?,
        })
    }

    fn send_packet_to_decoder(&mut self, packet: &Packet) {
        self.decoder.send_packet(packet).unwrap();
    }

    fn send_eof_to_decoder(&mut self) {
        self.decoder.send_eof().unwrap();
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

    fn send_frame_to_encoder(&mut self, frame: &frame::Video) {
        self.encoder.send_frame(frame).unwrap();
    }

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

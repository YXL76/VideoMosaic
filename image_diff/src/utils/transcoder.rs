use ffmpeg::{codec, decoder, encoder, format, frame, picture, Dictionary, Packet, Rational};

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

use ffmpeg::{codec, encoder, format, media, Rational};

fn main() {
    ffmpeg::init().unwrap();

    let input = std::env::args().nth(1).unwrap();
    let output = std::env::args().nth(2).unwrap();

    let mut input = format::input(&input).unwrap();
    let mut output = format::output(&output).unwrap();

    let mut stream_mapping = vec![0; input.nb_streams() as _];
    let mut ist_time_bases = vec![Rational(0, 1); input.nb_streams() as _];
    let mut ost_index = 0;

    for (ist_index, is) in input.streams().enumerate() {
        let ist_medium = is.parameters().medium();
        if ist_medium != media::Type::Audio
            && ist_medium != media::Type::Video
            && ist_medium != media::Type::Subtitle
        {
            stream_mapping[ist_index] = -1;
            continue;
        }
        stream_mapping[ist_index] = ost_index;
        ist_time_bases[ist_index] = is.time_base();
        ost_index += 1;
        let imeta = is.metadata();

        let mut os = output.add_stream(encoder::find(codec::Id::None)).unwrap();
        os.set_parameters(is.parameters());

        let mut ometa = ffmpeg::Dictionary::new();

        if let Some(lang) = imeta.get("language") {
            ometa.set("language", lang);
        }

        if ist_medium == media::Type::Subtitle {
            if let Some(title) = imeta.get("title") {
                ometa.set("title", title);
            }
        }

        os.set_metadata(ometa);
    }

    for chapter in input.chapters() {
        output
            .add_chapter(
                chapter.id(),
                chapter.time_base(),
                chapter.start(),
                chapter.end(),
                chapter.metadata().get("title").unwrap(),
            )
            .unwrap();
    }

    output.set_metadata(ffmpeg::Dictionary::new());
    output.write_header().unwrap();

    for (stream, mut packet) in input.packets() {
        let ist_index = stream.index();
        let ost_index = stream_mapping[ist_index];
        if ost_index < 0 {
            continue;
        }
        let ost = output.stream(ost_index as _).unwrap();
        packet.rescale_ts(ist_time_bases[ist_index], ost.time_base());
        packet.set_position(-1);
        packet.set_stream(ost_index as _);
        packet.write_interleaved(&mut output).unwrap();
    }

    output.write_trailer().unwrap();
}

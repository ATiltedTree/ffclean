use ffmpeg::{codec, encoder, format, media};

fn main() {
    ffmpeg::init().unwrap();

    let input = std::env::args().nth(1).unwrap();
    let output = std::env::args().nth(2).unwrap();

    let mut input = format::input(&input).unwrap();
    let mut output = format::output(&output).unwrap();

    for is in input.streams() {
        let imedium = is.parameters().medium();
        let imeta = is.metadata();

        let mut os = output.add_stream(encoder::find(codec::Id::None)).unwrap();
        os.set_parameters(is.parameters());

        let mut ometa = ffmpeg::Dictionary::new();

        macro_rules! map_meta {
            ($meta:literal) => {
                if let Some(meta) = imeta.get($meta) {
                    ometa.set($meta, meta);
                }
            };
        }

        map_meta!("language");

        if imedium == media::Type::Subtitle {
            map_meta!("title");
        }

        if imedium == media::Type::Attachment {
            map_meta!("filename");
            map_meta!("mimetype");
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
                chapter.metadata().get("title").unwrap_or_default(),
            )
            .unwrap();
    }

    output.set_metadata(ffmpeg::Dictionary::new());
    output.write_header().unwrap();

    for (stream, mut packet) in input.packets() {
        packet.set_position(-1);
        packet.set_stream(stream.index());
        packet.write_interleaved(&mut output).unwrap();
    }

    output.write_trailer().unwrap();
}

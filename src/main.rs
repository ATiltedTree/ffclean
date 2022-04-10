use ffmpeg::{codec, encoder, format, media};

fn main() {
    ffmpeg::init().unwrap();

    let input = std::env::args().nth(1).unwrap();
    let output = std::env::args().nth(2).unwrap();

    let mut input = format::input(&input).unwrap();
    let mut output = format::output(&output).unwrap();

    for is in input.streams() {
        let ist_medium = is.parameters().medium();
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

        if ist_medium == media::Type::Attachment {
            if let Some(filename) = imeta.get("filename") {
                ometa.set("filename", filename);
            }

            if let Some(mimetype) = imeta.get("mimetype") {
                ometa.set("mimetype", mimetype);
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
                chapter.metadata().get("title").unwrap_or_default(),
            )
            .unwrap();
    }

    output.set_metadata(ffmpeg::Dictionary::new());
    output.write_header().unwrap();

    for (stream, mut packet) in input.packets() {
        let ist_index = stream.index();
        let ost = output.stream(ist_index as _).unwrap();
        packet.set_position(-1);
        packet.set_stream(ist_index as _);
        packet.write_interleaved(&mut output).unwrap();
    }

    output.write_trailer().unwrap();
}

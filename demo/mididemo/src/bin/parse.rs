use midly::Smf;

fn main() {
    let smf = Smf::parse(include_bytes!("./../../MIDI_sample.mid")).unwrap();

    println!("{:?}", smf.header);
    for (i, track) in smf.tracks.iter().enumerate() {
        println!("track {} has {} events", i, track.len());
        if i <= 1 {
            for ev in track.iter() {
                println!("{:?}", ev);
            }
        }
    }
}

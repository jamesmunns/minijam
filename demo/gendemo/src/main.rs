use std::{thread::sleep, time::Duration};

use minijam::scale::{Pitch, MAJOR_PENTATONIC_INTERVALS};
use thursday::{phrdat::{PhraseDataParameters, PhraseDataBuilder}, Length};
use rand::thread_rng;

fn main() {
    let mut rng = thread_rng();
    let params = PhraseDataParameters::default();
    let mut builder = PhraseDataBuilder::default();
    // for _ in 0..10 {
    //     builder.fill(&mut rng, &params);
    //     println!("{:#?}", builder);
    //     sleep(Duration::from_millis(2000));
    // }
    builder.fill(&mut rng, &params);
    println!("{:#?}", builder);

    let mut bbs = vec![];
    for (i, lead) in builder.lead_voices.iter().enumerate() {
        let mut bb = BarBuf::new();
        let mut lend = 0;
        assert_ne!(0, lead.rhythm.len());
        for note in lead.rhythm.iter() {
            let start = note.ppqn_start();
            if start > lend {
                let delta = start - lend;
                bb.push_rest_simple(Length::PPQNCount(delta)).unwrap();
                lend += delta;
            }
            bb.push_note_simple(
                Length::PPQNCount(note.ppqn_len()),
                ((Pitch::C as u8) + MAJOR_PENTATONIC_INTERVALS[i].0 as u8).into(),
                5
            ).unwrap();
            lend += note.ppqn_len();
        }
        bbs.push(bb)
    }
    for (i, chorus) in builder.chorus_voices.iter().enumerate() {
        let mut bb = BarBuf::new();
        let mut lend = 0;
        assert_ne!(0, chorus.rhythm.len());
        for note in chorus.rhythm.iter() {
            let start = note.ppqn_start();
            if start > lend {
                let delta = start - lend;
                bb.push_rest_simple(Length::PPQNCount(delta)).unwrap();
                lend += delta;
            }
            bb.push_note_simple(
                Length::PPQNCount(note.ppqn_len()),
                ((Pitch::C as u8) + MAJOR_PENTATONIC_INTERVALS[i].0 as u8).into(),
                3
            ).unwrap();
            lend += note.ppqn_len();
        }
        bbs.push(bb)
    }

    let btm = bbs.iter().map(|bar| (bar, None)).collect::<Vec<_>>();
    println!("{}", btm.len());
    println!("{}, {}", builder.lead_voices.len(), builder.chorus_voices.len());
    bars_to_midi(&btm, "rythm.mid", builder.build_header().bpm as u32).unwrap();

}

use std::error::Error;

use midly::{Header, Smf, Format, TrackEvent, TrackEventKind, MetaMessage, MidiMessage};
use thursday::{bars::BarBuf, PPQN};

pub fn bar_to_midi(
    bbuf: &BarBuf,
    path: &str,
    bpm: u32,
    midi_instrument: Option<u8>,
) -> Result<(), Box<dyn Error>> {
    bars_to_midi(&[(bbuf, midi_instrument)], path, bpm)?;
    Ok(())
}

pub fn bars_to_midi(
    bbufs: &[(&BarBuf, Option<u8>)],
    path: &str,
    bpm: u32,
) -> Result<(), Box<dyn Error>> {
    let mut smf = Smf::new(Header::new(
        Format::Parallel,
        midly::Timing::Metrical(PPQN.into()),
    ));
    let mut track_0: Vec<TrackEvent> = vec![];

    // Fixed metadata
    track_0.push(TrackEvent {
        delta: 0u32.into(),
        kind: TrackEventKind::Meta(MetaMessage::TrackName(b"thursday")),
    });
    track_0.push(TrackEvent {
        delta: 0u32.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(((1_000_000u32 * 60) / bpm).into())),
    });
    track_0.push(TrackEvent {
        delta: 0u32.into(),
        kind: TrackEventKind::Meta(MetaMessage::TimeSignature(4, 2, 24, 8)),
    });
    track_0.push(TrackEvent {
        delta: 0u32.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    smf.tracks.push(track_0);

    for (i, (bbuf, instr)) in bbufs.iter().enumerate() {
        let ch = (i as u8).into();
        let mut track_n: Vec<TrackEvent> = vec![];

        track_n.push(TrackEvent {
            delta: 0u32.into(),
            kind: TrackEventKind::Midi {
                channel: ch,
                message: MidiMessage::Controller {
                    controller: 0u8.into(),
                    value: 121u8.into(),
                },
            },
        });
        track_n.push(TrackEvent {
            delta: 0u32.into(),
            kind: TrackEventKind::Midi {
                channel: ch,
                message: MidiMessage::Controller {
                    controller: 32u8.into(),
                    value: 0u8.into(),
                },
            },
        });
        track_n.push(TrackEvent {
            delta: 0u32.into(),
            kind: TrackEventKind::Midi {
                channel: ch,
                message: MidiMessage::ProgramChange {
                    // Default to piano
                    program: instr.unwrap_or(1).into(),
                },
            },
        });


        let mut idx = 0u16;
        for note in bbuf.notes() {
            track_n.push(TrackEvent {
                delta: ((note.ppqn_start() - idx) as u32).into(),
                kind: TrackEventKind::Midi {
                    channel: ch,
                    message: MidiMessage::NoteOn {
                        key: note.pitch_tone_offset().0.into(),
                        vel: 80u8.into(),
                    },
                },
            });
            track_n.push(TrackEvent {
                delta: (note.ppqn_len() as u32).into(),
                kind: TrackEventKind::Midi {
                    channel: ch,
                    message: MidiMessage::NoteOff {
                        key: note.pitch_tone_offset().0.into(),
                        vel: 64u8.into(),
                    },
                },
            });
            idx = note.ppqn_start() + note.ppqn_len();
        }

        track_n.push(TrackEvent {
            delta: 0u32.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });

        smf.tracks.push(track_n);
    }

    println!("!!! {} !!!", smf.tracks.len());

    smf.save(path)?;

    Ok(())
}

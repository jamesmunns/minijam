use midly::{Format, Header, MetaMessage, MidiMessage, Smf, TrackEvent, TrackEventKind};
use thursday::{PPQN, PPQN_EIGHTH};

pub fn main() {
    let mut smf = Smf::new(Header::new(
        Format::Parallel,
        midly::Timing::Metrical(PPQN.into()),
    ));
    let mut track_0: Vec<TrackEvent> = vec![];
    let mut track_1: Vec<TrackEvent> = vec![];

    // Set tempo (120bpm)
    track_0.push(TrackEvent {
        delta: 0u32.into(),
        kind: TrackEventKind::Meta(MetaMessage::TrackName(b"hello")),
    });
    track_0.push(TrackEvent {
        delta: 0u32.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo((1_000_000u32 / (120 / 60)).into())),
    });
    track_0.push(TrackEvent {
        delta: 0u32.into(),
        kind: TrackEventKind::Meta(MetaMessage::TimeSignature(4, 2, 24, 8)),
    });
    track_0.push(TrackEvent {
        delta: 0u32.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    track_1.push(TrackEvent {
        delta: 0u32.into(),
        kind: TrackEventKind::Meta(MetaMessage::TrackName(b"world")),
    });

    track_1.push(TrackEvent {
        delta: 0u32.into(),
        kind: TrackEventKind::Midi {
            channel: 0u8.into(),
            message: MidiMessage::Controller {
                controller: 0u8.into(),
                value: 121u8.into(),
            },
        },
    });
    track_1.push(TrackEvent {
        delta: 0u32.into(),
        kind: TrackEventKind::Midi {
            channel: 0u8.into(),
            message: MidiMessage::Controller {
                controller: 32u8.into(),
                value: 0u8.into(),
            },
        },
    });
    track_1.push(TrackEvent {
        delta: 0u32.into(),
        kind: TrackEventKind::Midi {
            channel: 0u8.into(),
            message: MidiMessage::ProgramChange {
                program: 1u8.into(),
            },
        },
    });

    for note in 60u8..72 {
        // on
        track_1.push(TrackEvent {
            delta: 0u32.into(),
            kind: TrackEventKind::Midi {
                channel: 0u8.into(),
                message: MidiMessage::NoteOn {
                    key: note.into(),
                    vel: 80u8.into(),
                },
            },
        });
        track_1.push(TrackEvent {
            delta: (PPQN_EIGHTH as u32).into(),
            kind: TrackEventKind::Midi {
                channel: 1u8.into(),
                message: MidiMessage::NoteOff {
                    key: note.into(),
                    vel: 64u8.into(),
                },
            },
        });
    }

    track_1.push(TrackEvent {
        delta: 0u32.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    smf.tracks.push(track_0);
    smf.tracks.push(track_1);

    smf.save("poc.mid").unwrap();
}

use core::ops::DerefMut;

use minijam::{Track, StereoSample, Sample, scale::{Pitch, Note, NATURAL_MAJOR_INTERVALS, MAJOR_TRIAD_INTERVALS, MINOR_TRIAD_INTERVALS, Semitones}, tones::ToneKind};
// use userspace::common::porcelain::{
//     pcm_sink as pcm,
//     time,
//     system,
// };
use rand::{thread_rng, RngCore, prelude::ThreadRng};
use wav::{Header, WAV_FORMAT_PCM, BitDepth};

const CHUNK_SZ: usize = 512;

pub fn main() {
    let mut rng = thread_rng();
    let mut track_lead1: Track<128> = Track::new(44100);
    let mut track_lead2: Track<128> = Track::new(44100);
    let mut track_ch1: Track<128> = Track::new(44100);
    let mut track_ch2: Track<128> = Track::new(44100);
    let mut track_ch3: Track<128> = Track::new(44100);

    let mut all_samples: Vec<StereoSample> = Vec::with_capacity(44100 * 180);

    loop {
        track_lead1.reset();
        track_lead2.reset();
        track_ch1.reset();
        track_ch2.reset();
        track_ch3.reset();

        let pitch = match rng.next_u32() % 12 {
            0 => Pitch::C,
            1 => Pitch::CSharp,
            2 => Pitch::D,
            3 => Pitch::DSharp,
            4 => Pitch::E,
            5 => Pitch::F,
            6 => Pitch::FSharp,
            7 => Pitch::G,
            8 => Pitch::GSharp,
            9 => Pitch::A,
            10 => Pitch::ASharp,
            _ => Pitch::B,
        };

        let scales = &[
            minijam::scale::IONIAN_INTERVALS,
            // minijam::scale::DORIAN_INTERVALS,
            // minijam::scale::PHRYGIAN_INTERVALS,
            minijam::scale::LYDIAN_INTERVALS,
            minijam::scale::MIXOLYDIAN_INTERVALS,
            // minijam::scale::AEOLIAN_INTERVALS,
            // minijam::scale::LOCRIAN_INTERVALS,
            // minijam::scale::HARMONIC_MINOR_INTERVALS,
            // minijam::scale::MELODIC_MINOR_ASCENDING_INTERVALS,
            // minijam::scale::MELODIC_MINOR_DESCENDING_INTERVALS,
            minijam::scale::MAJOR_TRIAD_INTERVALS,
            // minijam::scale::MINOR_TRIAD_INTERVALS,
            // minijam::scale::DIMINISHED_TRIAD_INTERVALS,
            // minijam::scale::AUGMENTED_TRIAD_INTERVALS,
            minijam::scale::DOMINANT_7TH_TETRAD_INTERVALS,
            // minijam::scale::MINOR_7TH_TETRAD_INTERVALS,
            minijam::scale::MAJOR_7TH_TETRAD_INTERVALS,
            // minijam::scale::MINOR_MAJOR_7TH_TETRAD_INTERVALS,
            // minijam::scale::AUGMENTED_7TH_TETRAD_INTERVALS,
            minijam::scale::AUGMENTED_MAJOR_7TH_TETRAD_INTERVALS,
            minijam::scale::DIMINISHED_7TH_TETRAD_INTERVALS,
            // minijam::scale::DIMINISHED_HALF_7TH_TETRAD_INTERVALS,
            minijam::scale::MAJOR_PENTATONIC_INTERVALS,
            minijam::scale::EGYPTIAN_PENTATONIC_INTERVALS,
            // minijam::scale::BLUES_MINOR_PENTATONIC_INTERVALS,
            minijam::scale::BLUES_MAJOR_PENTATONIC_INTERVALS,
            // minijam::scale::MINOR_PENTATONIC_INTERVALS,
        ];

        let scale = scales[rng.next_u32() as usize % scales.len()];

        let major_chords = &[
            (NATURAL_MAJOR_INTERVALS[0], MAJOR_TRIAD_INTERVALS), // I - Primary
            (NATURAL_MAJOR_INTERVALS[1], MINOR_TRIAD_INTERVALS), // II
            (NATURAL_MAJOR_INTERVALS[2], MINOR_TRIAD_INTERVALS), // III
            (NATURAL_MAJOR_INTERVALS[3], MAJOR_TRIAD_INTERVALS), // IV - Primary
            (NATURAL_MAJOR_INTERVALS[4], MAJOR_TRIAD_INTERVALS), // V - Primary
            (NATURAL_MAJOR_INTERVALS[5], MINOR_TRIAD_INTERVALS), // VI
        ];


        let wave = match rng.next_u32() % 3 {
            0 => minijam::tones::ToneKind::Sine,
            1 => minijam::tones::ToneKind::Square,
            _ => minijam::tones::ToneKind::Saw,
        };
        // let wave = minijam::tones::ToneKind::Sine;

        let bpm = 180;
        let full_length = (44100 * 60) / bpm;
        let note_length = (full_length * 9) / 10;
        let scale_len = scale.len();
        let mut cur = 0;

        for _ in 0..64 {
            let chance = rng.next_u32();
            if chance < 0xD000_0000 {
                let oct = match rng.next_u32() % 3 {
                    0 => 2,
                    1 => 3,
                    _ => 4,
                };

                let offset = scale[rng.next_u32() as usize % scale_len];
                let note = Note {
                    pitch,
                    octave: oct,
                };
                let note = note + offset;

                track_lead1.add_note(wave, note, cur, cur + note_length).unwrap();
            }
            cur += full_length;
        }

        let full_length_2 = ((44100 * 60) / bpm) / 2;
        let note_length_2 = (full_length_2 * 9) / 10;
        let mut cur_2 = 0;

        let wave2 = match rng.next_u32() % 3 {
            0 => minijam::tones::ToneKind::Sine,
            1 => minijam::tones::ToneKind::Square,
            _ => minijam::tones::ToneKind::Saw,
        };

        for _ in 0..128 {
            let chance = rng.next_u32();
            if chance < 0x8000_0000 {
                let oct = match rng.next_u32() % 3 {
                    0 => 2,
                    1 => 3,
                    _ => 4,
                };

                let offset = scale[rng.next_u32() as usize % scale_len];
                let note = Note {
                    pitch,
                    octave: oct,
                };
                let note = note + offset;

                track_lead2.add_note(wave2, note, cur_2, cur_2 + note_length_2).unwrap();
            }
            cur_2 += full_length_2;
        }

        // Chords
        // First and Last should be I
        let chord_wave = match rng.next_u32() % 3 {
            0 => minijam::tones::ToneKind::Sine,
            1 => minijam::tones::ToneKind::Square,
            _ => minijam::tones::ToneKind::Saw,
        };
        let ch_full_length = ((44100 * 60) / bpm) * 4;
        let ch_note_length = (ch_full_length * 9) / 10;
        let mut ch_cur = 0;
        let ch_note = Note {
            pitch,
            octave: 3,
        };

        fill_chord(
            &mut [&mut track_ch1, &mut track_ch2, &mut track_ch3],
            &mut rng,
            0xC000_0000, // 75%
            ch_note,
            &[major_chords[0]],
            chord_wave,
            &mut ch_cur,
            ch_note_length,
            ch_full_length
        );

        ch_cur += ch_full_length;

        for _ in 0..13 {
            fill_chord(
                &mut [&mut track_ch1, &mut track_ch2, &mut track_ch3],
                &mut rng,
                0xC000_0000, // 75%
                ch_note,
                major_chords,
                chord_wave,
                &mut ch_cur,
                ch_note_length,
                ch_full_length
            );
        }

        fill_chord(
            &mut [&mut track_ch1, &mut track_ch2, &mut track_ch3],
            &mut rng,
            0xC000_0000, // 75%
            ch_note,
            &[major_chords[3], major_chords[4]],
            chord_wave,
            &mut ch_cur,
            ch_note_length,
            ch_full_length
        );

        fill_chord(
            &mut [&mut track_ch1, &mut track_ch2, &mut track_ch3],
            &mut rng,
            0xC000_0000, // 75%
            ch_note,
            &[major_chords[0]],
            chord_wave,
            &mut ch_cur,
            ch_note_length,
            ch_full_length
        );

        // End chords

        while ![&track_lead1, &track_lead2, &track_ch1, &track_ch2, &track_ch3].iter().all(|t| t.is_done()) {
            let empty_samp = Sample { word: 0 };
            let mut samples = vec![StereoSample { left: empty_samp, right: empty_samp }; 512];

            track_lead1.fill_stereo_samples(&mut samples, minijam::tones::Mix::Div4);
            track_lead2.fill_stereo_samples(&mut samples, minijam::tones::Mix::Div4);
            track_ch1.fill_stereo_samples(&mut samples, minijam::tones::Mix::Div8);
            track_ch2.fill_stereo_samples(&mut samples, minijam::tones::Mix::Div8);
            track_ch3.fill_stereo_samples(&mut samples, minijam::tones::Mix::Div8);

            all_samples.extend_from_slice(&samples);
        }

        if all_samples.len() >= (44100 * 180) {
            break;
        }
    }

    use std::fs::File;
    use std::path::Path;

    let header = Header::new(
        WAV_FORMAT_PCM,
        2,
        44100,
        16
    );

    let mut new_data: Vec<i16> = Vec::new();
    for samp in all_samples.iter() {
        new_data.push(unsafe { samp.left.word });
        new_data.push(unsafe { samp.right.word });
    }

    let mut out_file = File::create(Path::new("target/jam.wav")).unwrap();
    wav::write(header, &BitDepth::Sixteen(new_data), &mut out_file).unwrap();
}

fn fill_chord<const N: usize>(
    tracks: &mut [&mut Track<N>],
    rng: &mut ThreadRng,
    prob: u32,
    note: Note,
    chords: &[(Semitones, &[Semitones])],
    kind: ToneKind,
    cur_samp: &mut u32,
    note_len: u32,
    total_len: u32,
) {
    let ch = if chords.len() > 1 {
        chords[rng.next_u32() as usize % chords.len()]
    } else {
        chords[0]
    };

    for (track, semi) in tracks.iter_mut().zip(ch.1.iter()) {
        let chance = rng.next_u32();
        if chance < prob {
            let note = note + ch.0 + *semi;
            track.add_note(kind, note, *cur_samp, *cur_samp + note_len).unwrap();
        }
    }

    *cur_samp += total_len;
}

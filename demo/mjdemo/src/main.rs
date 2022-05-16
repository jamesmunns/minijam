use core::ops::DerefMut;

use minijam::{Track, StereoSample, Sample, scale::{Pitch, Note, NATURAL_MAJOR_INTERVALS, MAJOR_TRIAD_INTERVALS, MINOR_TRIAD_INTERVALS, Semitones, NATURAL_MINOR_INTERVALS, DIMINISHED_TRIAD_INTERVALS}, tones::ToneKind};
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

    let mut lead_voice_1 = match rng.next_u32() % 3 {
        0 => minijam::tones::ToneKind::Sine,
        1 => minijam::tones::ToneKind::Square,
        _ => minijam::tones::ToneKind::Saw,
    };

    let mut lead_voice_2 = match rng.next_u32() % 3 {
        0 => minijam::tones::ToneKind::Sine,
        1 => minijam::tones::ToneKind::Square,
        _ => minijam::tones::ToneKind::Saw,
    };

    let mut chorus_voice = match rng.next_u32() % 3 {
        0 => minijam::tones::ToneKind::Sine,
        1 => minijam::tones::ToneKind::Square,
        _ => minijam::tones::ToneKind::Saw,
    };

    let mut key = match rng.next_u32() % 12 {
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

    let mut bpm = 180;

    let major_scales = &[
        minijam::scale::IONIAN_INTERVALS,
        minijam::scale::LYDIAN_INTERVALS,
        minijam::scale::MIXOLYDIAN_INTERVALS,
        minijam::scale::MAJOR_TRIAD_INTERVALS,
        minijam::scale::DOMINANT_7TH_TETRAD_INTERVALS,
        minijam::scale::MAJOR_7TH_TETRAD_INTERVALS,
        minijam::scale::AUGMENTED_MAJOR_7TH_TETRAD_INTERVALS,
        minijam::scale::DIMINISHED_7TH_TETRAD_INTERVALS,
        minijam::scale::MAJOR_PENTATONIC_INTERVALS,
        minijam::scale::EGYPTIAN_PENTATONIC_INTERVALS,
        minijam::scale::BLUES_MAJOR_PENTATONIC_INTERVALS,
    ];

    let minor_scales = &[
        minijam::scale::DORIAN_INTERVALS,
        minijam::scale::PHRYGIAN_INTERVALS,
        minijam::scale::AEOLIAN_INTERVALS,
        minijam::scale::LOCRIAN_INTERVALS,
        minijam::scale::HARMONIC_MINOR_INTERVALS,
        minijam::scale::MELODIC_MINOR_ASCENDING_INTERVALS,
        minijam::scale::MELODIC_MINOR_DESCENDING_INTERVALS,
        minijam::scale::MINOR_TRIAD_INTERVALS,
        minijam::scale::DIMINISHED_TRIAD_INTERVALS,
        minijam::scale::AUGMENTED_TRIAD_INTERVALS,
        minijam::scale::MINOR_7TH_TETRAD_INTERVALS,
        minijam::scale::MINOR_MAJOR_7TH_TETRAD_INTERVALS,
        minijam::scale::AUGMENTED_7TH_TETRAD_INTERVALS,
        minijam::scale::DIMINISHED_HALF_7TH_TETRAD_INTERVALS,
        minijam::scale::BLUES_MINOR_PENTATONIC_INTERVALS,
        minijam::scale::MINOR_PENTATONIC_INTERVALS,
    ];

    let major_chords = &[
        (NATURAL_MAJOR_INTERVALS[0], MAJOR_TRIAD_INTERVALS), // I - Primary
        (NATURAL_MAJOR_INTERVALS[1], MINOR_TRIAD_INTERVALS), // II
        (NATURAL_MAJOR_INTERVALS[2], MINOR_TRIAD_INTERVALS), // III
        (NATURAL_MAJOR_INTERVALS[3], MAJOR_TRIAD_INTERVALS), // IV - Primary
        (NATURAL_MAJOR_INTERVALS[4], MAJOR_TRIAD_INTERVALS), // V - Primary
        (NATURAL_MAJOR_INTERVALS[5], MINOR_TRIAD_INTERVALS), // VI
    ];

    let minor_chords = &[
        (NATURAL_MINOR_INTERVALS[0], MINOR_TRIAD_INTERVALS), // I
        (NATURAL_MINOR_INTERVALS[1], DIMINISHED_TRIAD_INTERVALS), // II
        (NATURAL_MINOR_INTERVALS[2], MINOR_TRIAD_INTERVALS),      // III
        (NATURAL_MINOR_INTERVALS[3], MINOR_TRIAD_INTERVALS), // IV
        (NATURAL_MINOR_INTERVALS[4], MAJOR_TRIAD_INTERVALS), // V
        (NATURAL_MINOR_INTERVALS[5], MINOR_TRIAD_INTERVALS),      // VI
    ];

    let mut scales = major_scales.as_slice();
    let mut chords = major_chords;

    let mut scale = scales[rng.next_u32() as usize % scales.len()];
    let mut scale_len = scale.len();

    let mut lead_1_chance = rng.next_u32().min(0xC000_0000).max(0x2000_0000);
    let mut lead_2_chance = rng.next_u32().min(0xC000_0000).max(0x2000_0000);
    let mut chorus_chance = rng.next_u32().min(0xC000_0000).max(0x6000_0000);

    let mut lead_1_refrain: Vec<Option<Note>> = Vec::new();
    let mut lead_2_refrain: Vec<Option<Note>> = Vec::new();
    let mut chorus_refrain: [Vec<Option<Note>>; 3] = [Vec::new(), Vec::new(), Vec::new()];


    // Seed stuff
    {
        lead_1_refrain.clear();
        let to_gen = (rng.next_u32() % 32) + 1;
        let chance = rng.next_u32();
        for _ in 0..to_gen {
            if chance < lead_1_chance {
                let oct = match rng.next_u32() % 3 {
                    0 => 2,
                    1 => 3,
                    _ => 4,
                };

                let offset = scale[rng.next_u32() as usize % scale.len()];
                let note = Note {
                    pitch: key,
                    octave: oct,
                };
                let note = note + offset;
                lead_1_refrain.push(Some(note)); //.ok();
            } else {
                lead_1_refrain.push(None); //.ok();
            }
        }
    }

    {
        lead_2_refrain.clear();
        let to_gen = (rng.next_u32() % 64) + 1;
        let chance = rng.next_u32();
        for _ in 0..to_gen {
            if chance < lead_2_chance {
                let oct = match rng.next_u32() % 3 {
                    0 => 2,
                    1 => 3,
                    _ => 4,
                };

                let offset = scale[rng.next_u32() as usize % scale.len()];
                let note = Note {
                    pitch: key,
                    octave: oct,
                };
                let note = note + offset;
                lead_2_refrain.push(Some(note)); //.ok();
            } else {
                lead_2_refrain.push(None); //.ok();
            }
        }
    }

    {
        let ch_note = Note {
            pitch: key,
            octave: 3,
        };
        chorus_refrain.iter_mut().for_each(|v| v.clear());
        fill_chord2(
            &mut chorus_refrain,
            &mut rng,
            chorus_chance,
            ch_note,
            &[chords[0]],
        );

        for _ in 0..5 {
            fill_chord2(
                &mut chorus_refrain,
                &mut rng,
                chorus_chance,
                ch_note,
                chords,
            );
        }

        fill_chord2(
            &mut chorus_refrain,
            &mut rng,
            chorus_chance,
            ch_note,
            &[chords[3], chords[4]],
        );

        fill_chord2(
            &mut chorus_refrain,
            &mut rng,
            chorus_chance,
            ch_note,
            &[chords[0]],
        );
    }


    loop {
        track_lead1.reset();
        track_lead2.reset();
        track_ch1.reset();
        track_ch2.reset();
        track_ch3.reset();

        match rng.next_u32() % 13 {
            0 => {
                lead_voice_1 = match rng.next_u32() % 3 {
                    0 => minijam::tones::ToneKind::Sine,
                    1 => minijam::tones::ToneKind::Square,
                    _ => minijam::tones::ToneKind::Saw,
                };
            }
            1 => {
                lead_voice_2 = match rng.next_u32() % 3 {
                    0 => minijam::tones::ToneKind::Sine,
                    1 => minijam::tones::ToneKind::Square,
                    _ => minijam::tones::ToneKind::Saw,
                };
            }
            2 => {
                chorus_voice = match rng.next_u32() % 3 {
                    0 => minijam::tones::ToneKind::Sine,
                    1 => minijam::tones::ToneKind::Square,
                    _ => minijam::tones::ToneKind::Saw,
                };
            }
            3 => {
                scale = scales[rng.next_u32() as usize % scales.len()];
                scale_len = scale.len();
            }
            4 => {
                lead_1_chance = rng.next_u32().min(0xC000_0000).max(0x2000_0000);
            }
            5 => {
                lead_2_chance = rng.next_u32().min(0xC000_0000).max(0x2000_0000);
            }
            6 => {
                chorus_chance = rng.next_u32().min(0xC000_0000).max(0x6000_0000);
            }
            7 => {
                bpm = 120 + rng.next_u32() % 64;
            }
            8 => {
                let old_chords = chords.as_ptr();

                if (rng.next_u32() & 0b1) == 0 {
                    chords = major_chords;
                    scales = major_scales.as_slice();
                } else {
                    chords = minor_chords;
                    scales = minor_scales.as_slice();
                }

                // TODO: If we change minor/major, regenerate everything so we aren't
                // playing off. I could probably be smarter and transpose the progressions
                // rather than regenerate everything
                if chords.as_ptr() != old_chords {
                    {
                        lead_1_refrain.clear();
                        let to_gen = (rng.next_u32() % 32) + 1;
                        let chance = rng.next_u32();
                        for _ in 0..to_gen {
                            if chance < lead_1_chance {
                                let oct = match rng.next_u32() % 3 {
                                    0 => 2,
                                    1 => 3,
                                    _ => 4,
                                };

                                let offset = scale[rng.next_u32() as usize % scale_len];
                                let note = Note {
                                    pitch: key,
                                    octave: oct,
                                };
                                let note = note + offset;
                                lead_1_refrain.push(Some(note)); //.ok();
                            } else {
                                lead_1_refrain.push(None); //.ok();
                            }
                        }
                    }

                    {
                        lead_2_refrain.clear();
                        let to_gen = (rng.next_u32() % 64) + 1;
                        let chance = rng.next_u32();
                        for _ in 0..to_gen {
                            if chance < lead_2_chance {
                                let oct = match rng.next_u32() % 3 {
                                    0 => 2,
                                    1 => 3,
                                    _ => 4,
                                };

                                let offset = scale[rng.next_u32() as usize % scale_len];
                                let note = Note {
                                    pitch: key,
                                    octave: oct,
                                };
                                let note = note + offset;
                                lead_2_refrain.push(Some(note)); //.ok();
                            } else {
                                lead_2_refrain.push(None); //.ok();
                            }
                        }
                    }

                    {
                        let ch_note = Note {
                            pitch: key,
                            octave: 3,
                        };
                        chorus_refrain.iter_mut().for_each(|v| v.clear());
                        fill_chord2(
                            &mut chorus_refrain,
                            &mut rng,
                            chorus_chance,
                            ch_note,
                            &[chords[0]],
                        );

                        for _ in 0..5 {
                            fill_chord2(
                                &mut chorus_refrain,
                                &mut rng,
                                chorus_chance,
                                ch_note,
                                chords,
                            );
                        }

                        fill_chord2(
                            &mut chorus_refrain,
                            &mut rng,
                            chorus_chance,
                            ch_note,
                            &[chords[3], chords[4]],
                        );

                        fill_chord2(
                            &mut chorus_refrain,
                            &mut rng,
                            chorus_chance,
                            ch_note,
                            &[chords[0]],
                        );
                    }
                }
            }
            9 => {
                lead_1_refrain.clear();
                let to_gen = (rng.next_u32() % 32) + 1;
                let chance = rng.next_u32();
                for _ in 0..to_gen {
                    if chance < lead_1_chance {
                        let oct = match rng.next_u32() % 3 {
                            0 => 2,
                            1 => 3,
                            _ => 4,
                        };

                        let offset = scale[rng.next_u32() as usize % scale_len];
                        let note = Note {
                            pitch: key,
                            octave: oct,
                        };
                        let note = note + offset;
                        lead_1_refrain.push(Some(note)); // .ok();
                    } else {
                        lead_1_refrain.push(None); // .ok();
                    }
                }
            }
            10 => {
                lead_2_refrain.clear();
                let to_gen = (rng.next_u32() % 64) + 1;
                let chance = rng.next_u32();
                for _ in 0..to_gen {
                    if chance < lead_2_chance {
                        let oct = match rng.next_u32() % 3 {
                            0 => 2,
                            1 => 3,
                            _ => 4,
                        };

                        let offset = scale[rng.next_u32() as usize % scale_len];
                        let note = Note {
                            pitch: key,
                            octave: oct,
                        };
                        let note = note + offset;
                        lead_2_refrain.push(Some(note)); //.ok();
                    } else {
                        lead_2_refrain.push(None); //.ok();
                    }
                }
            }
            11 => {
                let ch_note = Note {
                    pitch: key,
                    octave: 3,
                };
                chorus_refrain.iter_mut().for_each(|v| v.clear());
                fill_chord2(
                    &mut chorus_refrain,
                    &mut rng,
                    chorus_chance,
                    ch_note,
                    &[chords[0]],
                );

                for _ in 0..5 {
                    fill_chord2(
                        &mut chorus_refrain,
                        &mut rng,
                        chorus_chance,
                        ch_note,
                        chords,
                    );
                }

                fill_chord2(
                    &mut chorus_refrain,
                    &mut rng,
                    chorus_chance,
                    ch_note,
                    &[chords[3], chords[4]],
                );

                fill_chord2(
                    &mut chorus_refrain,
                    &mut rng,
                    chorus_chance,
                    ch_note,
                    &[chords[0]],
                );
            }
            _ => {
                key = match rng.next_u32() % 12 {
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

                {
                    lead_1_refrain.clear();
                    let to_gen = (rng.next_u32() % 32) + 1;
                    let chance = rng.next_u32();
                    for _ in 0..to_gen {
                        if chance < lead_1_chance {
                            let oct = match rng.next_u32() % 3 {
                                0 => 2,
                                1 => 3,
                                _ => 4,
                            };

                            let offset = scale[rng.next_u32() as usize % scale_len];
                            let note = Note {
                                pitch: key,
                                octave: oct,
                            };
                            let note = note + offset;
                            lead_1_refrain.push(Some(note)); //.ok();
                        } else {
                            lead_1_refrain.push(None); //.ok();
                        }
                    }
                }

                {
                    lead_2_refrain.clear();
                    let to_gen = (rng.next_u32() % 64) + 1;
                    let chance = rng.next_u32();
                    for _ in 0..to_gen {
                        if chance < lead_2_chance {
                            let oct = match rng.next_u32() % 3 {
                                0 => 2,
                                1 => 3,
                                _ => 4,
                            };

                            let offset = scale[rng.next_u32() as usize % scale_len];
                            let note = Note {
                                pitch: key,
                                octave: oct,
                            };
                            let note = note + offset;
                            lead_2_refrain.push(Some(note)); //.ok();
                        } else {
                            lead_2_refrain.push(None); //.ok();
                        }
                    }
                }

                {
                    let ch_note = Note {
                        pitch: key,
                        octave: 3,
                    };
                    chorus_refrain.iter_mut().for_each(|v| v.clear());
                    fill_chord2(
                        &mut chorus_refrain,
                        &mut rng,
                        chorus_chance,
                        ch_note,
                        &[chords[0]],
                    );

                    for _ in 0..5 {
                        fill_chord2(
                            &mut chorus_refrain,
                            &mut rng,
                            chorus_chance,
                            ch_note,
                            chords,
                        );
                    }

                    fill_chord2(
                        &mut chorus_refrain,
                        &mut rng,
                        chorus_chance,
                        ch_note,
                        &[chords[3], chords[4]],
                    );

                    fill_chord2(
                        &mut chorus_refrain,
                        &mut rng,
                        chorus_chance,
                        ch_note,
                        &[chords[0]],
                    );
                }
            }
        }

        {
            let full_length = (44100 * 60) / bpm;
            let note_length = (full_length * 9) / 10;
            let mut cur = 0;

            let v1_len = lead_1_refrain.len();
            let v1_rnd = 32 - v1_len;

            for n in lead_1_refrain.iter() {
                if let Some(note) = n {
                    track_lead1.add_note(lead_voice_1, *note, cur, cur + note_length).unwrap();
                }
                cur += full_length;
            }

            for _ in 0..v1_rnd {
                let chance = rng.next_u32();
                if chance < lead_1_chance {
                    let oct = match rng.next_u32() % 3 {
                        0 => 2,
                        1 => 3,
                        _ => 4,
                    };

                    let offset = scale[rng.next_u32() as usize % scale_len];
                    let note = Note {
                        pitch: key,
                        octave: oct,
                    };
                    let note = note + offset;

                    track_lead1.add_note(lead_voice_1, note, cur, cur + note_length).unwrap();
                }
                cur += full_length;
            }
        }

        {
            let full_length_2 = ((44100 * 60) / bpm) / 2;
            let note_length_2 = (full_length_2 * 9) / 10;
            let mut cur_2 = 0;

            let v2_len = lead_2_refrain.len();
            let v2_rnd = 64 - v2_len;

            for n in lead_2_refrain.iter() {
                if let Some(note) = n {
                    track_lead2.add_note(lead_voice_2, *note, cur_2, cur_2 + note_length_2).unwrap();
                }
                cur_2 += full_length_2;
            }

            for _ in 0..v2_rnd {
                let chance = rng.next_u32();
                if chance < lead_2_chance {
                    let oct = match rng.next_u32() % 3 {
                        0 => 2,
                        1 => 3,
                        _ => 4,
                    };

                    let offset = scale[rng.next_u32() as usize % scale_len];
                    let note = Note {
                        pitch: key,
                        octave: oct,
                    };
                    let note = note + offset;

                    track_lead2.add_note(lead_voice_2, note, cur_2, cur_2 + note_length_2).unwrap();
                }
                cur_2 += full_length_2;
            }
        }

        // Chords
        // First and Last should be I
        {
            let ch_full_length = ((44100 * 60) / bpm) * 4;
            let ch_note_length = (ch_full_length * 9) / 10;

            for (refrain, track) in chorus_refrain.iter_mut().zip(&mut [&mut track_ch1, &mut track_ch2, &mut track_ch3]) {
                let mut ch_cur = 0;
                for note in refrain.iter() {
                    if let Some(note) = note {
                        track.add_note(chorus_voice, *note, ch_cur, ch_cur + ch_note_length).unwrap();
                    }
                    ch_cur += ch_full_length;
                }
            }
        }


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

fn fill_chord2(
    tracks: &mut [Vec<Option<Note>>],
    rng: &mut ThreadRng,
    prob: u32,
    note: Note,
    chords: &[(Semitones, &[Semitones])],
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
            track.push(Some(note)); // .ok();
        } else {
            track.push(None); // .ok();
        }
    }
}

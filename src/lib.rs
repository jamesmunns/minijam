#![no_std]

use heapless::Deque;
use scale::Pitch;
use tones::{Tone, Mix, ToneKind};

pub mod tones;
pub mod scale;

pub struct Track<const DEPTH: usize> {
    pub note_q: Deque<Note, DEPTH>,
    pub current: Option<Note>,
    pub cur_samp: u32,
    pub sample_rate: u32,
}

impl<const DEPTH: usize> Track<DEPTH> {
    #[inline]
    pub fn new(sample_rate: u32) -> Self {
        Self {
            note_q: Deque::new(),
            current: None,
            cur_samp: 0,
            sample_rate,
        }
    }

    pub fn reset(&mut self) {
        self.note_q.clear();
        self.current = None;
        self.cur_samp = 0;
    }

    #[inline]
    pub fn fill_stereo_samples(&mut self, samples: &mut [StereoSample], mix: Mix) {
        let samp_len = samples.len() as u32;
        let need_new = if let Some(mut note) = self.current.take() {
            if note.samp_end < self.cur_samp {
                true
            } else {
                if (self.cur_samp + samp_len) > note.samp_end {
                    note.wave.fill_last_stereo_samples(samples, mix);
                } else {
                    note.wave.fill_stereo_samples(samples, mix);
                    self.current = Some(note);
                }
                false
            }
        } else {
            true
        };

        if need_new {
            while let Some(mut note) = self.note_q.pop_front() {
                if note.samp_start < self.cur_samp {
                    note.wave.fill_first_stereo_samples(samples, mix);
                    self.current = Some(note);
                    break;
                } else {
                    self.note_q.push_front(note).ok();
                    break;
                }
            }
        }

        // let needs_new_after;
        // let samp_len = samples.len() as u32;

        // if let Some(mut note) = self.current.take() {
        //     // We have a current note. Is this the last sample batch?
        //     if (self.cur_samp + samp_len) >= note.samp_end {
        //         // How many samples to fade out? Min 64 to prevent a quick fade
        //         let outro = (note.samp_end - self.cur_samp).max(64);
        //         note.wave.fill_last_stereo_samples(samples, mix);
        //         needs_new_after = Some(outro)
        //     } else {
        //         note.wave.fill_stereo_samples(samples, mix);
        //         self.current = Some(note);
        //         needs_new_after = None;
        //     }
        // } else {
        //     needs_new_after = Some(0);
        // }

        // if let Some(after) = needs_new_after {
        //     if let Some(mut next) = self.note_q.pop_front() {
        //         // Do we have at least 64 samples to "fade up", including the time
        //         // necessary to finish the last sound?
        //         let start = next.samp_start.max(self.cur_samp + after);

        //         if start < (self.cur_samp + samp_len - 64) {
        //             // Yup! Figure out where to start...
        //             let offset = if next.samp_start > self.cur_samp {
        //                 (next.samp_start - self.cur_samp) as usize
        //             } else {
        //                 0
        //             };

        //             next.wave.fill_first_stereo_samples(&mut samples[offset..], mix);
        //             self.current = Some(next);
        //         } else {
        //             // Nope. Just put it back.
        //             self.note_q.push_front(next).ok();
        //         }
        //     }
        // }

        self.cur_samp += samp_len;
    }

    pub fn add_note(
        &mut self,
        kind: ToneKind,
        note: scale::Note,
        start: u32,
        end: u32
    ) -> Result<(), ()> {
        let freq = note.freq_f32();
        self.add_note_freq(kind, freq, start, end)
    }

    pub fn add_note_freq(
        &mut self,
        kind: ToneKind,
        freq: f32,
        start: u32,
        end: u32
    ) -> Result<(), ()> {
        if end <= start {
            return Err(());
        }

        if self.note_q.is_full() {
            return Err(());
        }

        if let Some(note) = self.note_q.pop_back() {
            let out_of_order = start < note.samp_end;
            self.note_q.push_back(note).ok();
            if out_of_order {
                return Err(());
            }
        }

        let tone = match kind {
            ToneKind::Sine => Tone::new_sine(freq, self.sample_rate),
            ToneKind::Square => Tone::new_square(freq, self.sample_rate),
            ToneKind::Saw => Tone::new_saw(freq, self.sample_rate),
        };

        self.note_q.push_back(Note {
            wave: tone,
            samp_start: start,
            samp_end: end,
        }).map_err(drop)
    }

    pub fn is_done(&self) -> bool {
        self.note_q.is_empty() && self.current.is_none()
    }
}

pub struct Note {
    pub wave: Tone,
    pub samp_start: u32,
    pub samp_end: u32,
}


#[repr(C)]
pub struct StereoSample {
    pub left: Sample,
    pub right: Sample,
}

#[repr(C)]
pub union Sample {
    pub bytes: [u8; 2],
    pub word: i16,
}

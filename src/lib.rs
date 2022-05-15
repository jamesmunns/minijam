#![no_std]

use heapless::Deque;
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
    pub fn fill_stereo_samples(&mut self, samples: &mut [StereoSample], mix: Mix) {
        let needs_new_after;
        let samp_len = samples.len() as u32;

        if let Some(mut note) = self.current.take() {
            // We have a current note. Is this the last sample batch?
            if (self.cur_samp + samp_len) >= note.samp_end {
                // How many samples to fade out? Min 64 to prevent a quick fade
                let outro = (note.samp_end - self.cur_samp).max(64);
                note.wave.fill_last_stereo_samples(samples, mix);
                needs_new_after = Some(outro)
            } else {
                note.wave.fill_stereo_samples(samples, mix);
                self.current = Some(note);
                needs_new_after = None;
            }
        } else {
            needs_new_after = Some(0);
        }

        if let Some(after) = needs_new_after {
            if let Some(mut next) = self.note_q.pop_front() {
                // Do we have at least 64 samples to "fade up", including the time
                // necessary to finish the last sound?
                let start = next.samp_start.max(self.cur_samp + after);

                if start < (self.cur_samp + samp_len - 64) {
                    // Yup! Figure out where to start...
                    let offset = if next.samp_start > self.cur_samp {
                        (next.samp_start - self.cur_samp) as usize
                    } else {
                        0
                    };

                    next.wave.fill_first_stereo_samples(&mut samples[offset..], mix);
                    self.current = Some(next);
                } else {
                    // Nope. Just put it back.
                    self.note_q.push_front(next).ok();
                }
            }
        }

        self.cur_samp += samp_len;
    }

    pub fn add_note_freq(
        &mut self,
        freq: f32,
        kind: ToneKind,
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

use crate::StereoSample;

pub const SINE_TABLE: [i16; 256] = [
    0, 804, 1608, 2410, 3212, 4011, 4808, 5602, 6393, 7179, 7962, 8739, 9512, 10278, 11039, 11793,
    12539, 13279, 14010, 14732, 15446, 16151, 16846, 17530, 18204, 18868, 19519, 20159, 20787,
    21403, 22005, 22594, 23170, 23731, 24279, 24811, 25329, 25832, 26319, 26790, 27245, 27683,
    28105, 28510, 28898, 29268, 29621, 29956, 30273, 30571, 30852, 31113, 31356, 31580, 31785,
    31971, 32137, 32285, 32412, 32521, 32609, 32678, 32728, 32757, 32767, 32757, 32728, 32678,
    32609, 32521, 32412, 32285, 32137, 31971, 31785, 31580, 31356, 31113, 30852, 30571, 30273,
    29956, 29621, 29268, 28898, 28510, 28105, 27683, 27245, 26790, 26319, 25832, 25329, 24811,
    24279, 23731, 23170, 22594, 22005, 21403, 20787, 20159, 19519, 18868, 18204, 17530, 16846,
    16151, 15446, 14732, 14010, 13279, 12539, 11793, 11039, 10278, 9512, 8739, 7962, 7179, 6393,
    5602, 4808, 4011, 3212, 2410, 1608, 804, 0, -804, -1608, -2410, -3212, -4011, -4808, -5602,
    -6393, -7179, -7962, -8739, -9512, -10278, -11039, -11793, -12539, -13279, -14010, -14732,
    -15446, -16151, -16846, -17530, -18204, -18868, -19519, -20159, -20787, -21403, -22005, -22594,
    -23170, -23731, -24279, -24811, -25329, -25832, -26319, -26790, -27245, -27683, -28105, -28510,
    -28898, -29268, -29621, -29956, -30273, -30571, -30852, -31113, -31356, -31580, -31785, -31971,
    -32137, -32285, -32412, -32521, -32609, -32678, -32728, -32757, -32767, -32757, -32728, -32678,
    -32609, -32521, -32412, -32285, -32137, -31971, -31785, -31580, -31356, -31113, -30852, -30571,
    -30273, -29956, -29621, -29268, -28898, -28510, -28105, -27683, -27245, -26790, -26319, -25832,
    -25329, -24811, -24279, -23731, -23170, -22594, -22005, -21403, -20787, -20159, -19519, -18868,
    -18204, -17530, -16846, -16151, -15446, -14732, -14010, -13279, -12539, -11793, -11039, -10278,
    -9512, -8739, -7962, -7179, -6393, -5602, -4808, -4011, -3212, -2410, -1608, -804,
];

#[derive(Debug, Copy, Clone)]
pub enum ToneKind {
    Sine,
    Square,
    Saw,
}

pub struct Tone {
    kind: ToneKind,
    cur_offset: i32,
    incr: i32,
}

#[derive(Clone, Copy)]
pub enum Mix {
    Div1,
    Div2,
    Div4,
    Div8,
}

pub enum OperatorKind {
    AmplitudeLfo(Tone),
    FrequencyLfo(Tone),
    None,
}

pub struct Operator {
    pub kind: OperatorKind,
}

/// Shifts volume between 50%-100%
#[inline]
fn volume_shift(samp: i16, vol: i16) -> i16 {
    let samp = samp as i32;
    let vol = vol as i32; // i16::MIN..=i16::MAX
    let vol = vol.wrapping_add((u16::MAX / 2).into()); // 0..=(i16::MAX * 2)
    let vol = (vol >> 7) + 512; // 512..=1024
    let samp = samp.wrapping_mul(vol);
    let samp = samp >> 10;
    samp as i16
}

impl Operator {
    fn operate(&mut self, samp: i16) -> i16 {
        match &mut self.kind {
            OperatorKind::AmplitudeLfo(op) => {
                let ops = op.next_sample();
                volume_shift(samp, ops)
            },
            OperatorKind::FrequencyLfo(_op) => todo!(),
            OperatorKind::None => samp,
        }
    }
}

// TODO: Add some kind of volume shift for higher frequencies?

impl Mix {
    #[inline]
    fn to_shift(&self) -> i16 {
        match self {
            Mix::Div1 => 0,
            Mix::Div2 => 1,
            Mix::Div4 => 2,
            Mix::Div8 => 3,
        }
    }
}

impl Tone {
    pub fn new_sine(freq: f32, sample_rate: u32) -> Self {
        let samp_per_cyc: f32 = (sample_rate as f32) / freq;
        let fincr = (SINE_TABLE.len() as f32) / samp_per_cyc;
        let incr = (((1 << 24) as f32) * fincr) as i32;

        Self {
            kind: ToneKind::Sine,
            cur_offset: 0,
            incr,
        }
    }

    pub fn new_square(freq: f32, sample_rate: u32) -> Self {
        let samp_per_cyc: f32 = (sample_rate as f32) / freq;
        let fincr = (u32::MAX as f32) / samp_per_cyc;
        let incr = fincr as i32;

        Self {
            kind: ToneKind::Square,
            cur_offset: 0,
            incr,
        }
    }

    pub fn new_saw(freq: f32, sample_rate: u32) -> Self {
        let samp_per_cyc: f32 = (sample_rate as f32) / freq;
        let fincr = (u32::MAX as f32) / samp_per_cyc;
        let incr = fincr as i32;

        Self {
            kind: ToneKind::Saw,
            cur_offset: 0,
            incr,
        }
    }

    #[inline]
    pub fn next_sample(&mut self) -> i16 {
        (self.next_sample_func())(self)
    }

    #[inline]
    fn next_sample_func(&mut self) -> fn(&'_ mut Tone) -> i16
    {
        match self.kind {
            ToneKind::Sine => Tone::next_sample_sine,
            ToneKind::Square => Tone::next_sample_square,
            ToneKind::Saw => Tone::next_sample_saw,
        }
    }

    #[inline]
    pub fn fill_first_stereo_samples(&mut self, samples: &mut [StereoSample], mix: Mix, operator: &mut Operator) {
        let next_sample = self.next_sample_func();
        let shift = mix.to_shift();

        // TODO: more gentle?
        let mut ct = 1;

        // Fade out in 1/8th volume steps over the course of this sample.
        samples.chunks_mut(samples.len() / 32).for_each(|ch| {
            ch.iter_mut().for_each(|s| {
                let samp = operator.operate(next_sample(self)) >> shift;
                let rsamp = samp as i32;
                let rsamp = rsamp.wrapping_mul(ct); // multiply by 1..=32;
                let rsamp = rsamp >> 5; // divide by 32
                let samp = rsamp as i16;

                unsafe {
                    s.left.word = s.left.word.wrapping_add(samp);
                    s.right.word = s.right.word.wrapping_add(samp);
                }
            });
            ct += 1;
        });
    }

    #[inline]
    pub fn fill_last_stereo_samples(&mut self, samples: &mut [StereoSample], mix: Mix, operator: &mut Operator) {
        let next_sample = self.next_sample_func();
        let shift = mix.to_shift();

        // TODO: more gentle?
        let mut ct = 32;

        // Fade out in 1/8th volume steps over the course of this sample.
        samples.chunks_mut(samples.len() / 32).for_each(|ch| {
            ch.iter_mut().for_each(|s| {
                let samp = operator.operate(next_sample(self)) >> shift;
                let rsamp = samp as i32;
                let rsamp = rsamp.wrapping_mul(ct); // multiply by 1..=32;
                let rsamp = rsamp >> 5; // divide by 32
                let samp = rsamp as i16;

                unsafe {
                    s.left.word = s.left.word.wrapping_add(samp);
                    s.right.word = s.right.word.wrapping_add(samp);
                }
            });
            ct -= 1;
        });
    }

    #[inline]
    pub fn fill_stereo_samples(&mut self, samples: &mut [StereoSample], mix: Mix, operator: &mut Operator) {
        let next_sample = self.next_sample_func();
        let shift = mix.to_shift();

        samples.iter_mut().for_each(|s| {
            let samp = operator.operate(next_sample(self)) >> shift;
            unsafe {
                s.left.word = s.left.word.wrapping_add(samp);
                s.right.word = s.right.word.wrapping_add(samp);
            }
        });
    }

    #[inline]
    pub fn next_sample_sine(&mut self) -> i16 {
        let val = (self.cur_offset) as u32;
        let idx_now = ((val >> 24) & 0xFF) as u8;
        let idx_nxt = idx_now.wrapping_add(1);
        let base_val = SINE_TABLE[idx_now as usize] as i32;
        let next_val = SINE_TABLE[idx_nxt as usize] as i32;

        // Distance to next value - perform 256 slot linear interpolation
        let off = ((val >> 16) & 0xFF) as i32; // 0..=255
        let cur_weight = base_val.wrapping_mul(256i32.wrapping_sub(off));
        let nxt_weight = next_val.wrapping_mul(off);
        let ttl_weight = cur_weight.wrapping_add(nxt_weight);
        let ttl_val = ttl_weight >> 8; // div 256
        let ttl_val = ttl_val as i16;

        self.cur_offset = self.cur_offset.wrapping_add(self.incr);

        ttl_val
    }

    #[inline]
    pub fn next_sample_square(&mut self) -> i16 {
        let ttl_val = if self.cur_offset >= 0 { i16::MAX } else { i16::MIN };
        self.cur_offset = self.cur_offset.wrapping_add(self.incr);
        ttl_val
    }

    #[inline]
    pub fn next_sample_saw(&mut self) -> i16 {
        let ttl_val = (self.cur_offset >> 16) as i16;
        self.cur_offset = self.cur_offset.wrapping_add(self.incr);
        ttl_val
    }
}

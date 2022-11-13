pub struct Euc32 {
    interval: u32,
    data: u32,
}

impl Euc32 {
    pub fn new(hits: u32, interval: u32) -> Option<Euc32> {
        if (interval == 0) || (hits > 32) || (interval > 32) || (hits > interval) {
            return None;
        }
        if hits == 0 {
            return Some(Euc32 { interval, data: 0 });
        }

        // We want the counter to *start* as if we just advanced from zero,
        // and tripped over the first interval. This is morally equivalent to:
        //
        // ```
        // let mut ctr = 0u32;
        // while ctr < interval {
        //     ctr += hits;
        // }
        // ```
        //
        // Thank you Josh Simmons for figuring out the math!
        let mut ctr = hits * (((interval - 1) / hits) + 1);

        let mut data = 0;
        for i in 0..interval {
            if ctr >= interval {
                data |= 1 << i;
                ctr -= interval;
            }
            ctr += hits;
        }

        Some(Euc32 { interval, data })
    }

    pub fn stringify(&self) -> String {
        let mut out = String::new();
        out += "[";
        for i in 0..self.interval {
            out += if (self.data & (1 << i)) != 0 { "x" } else { "." };
        }
        out += "]";
        out
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sevens() {
        println!();
        for i in 0..=7 {
            let x = Euc32::new(i, 7).unwrap();
            println!("{}: {}", i, x.stringify());
        }
    }

    #[test]
    fn thirteens() {
        println!();
        for i in 0..=13 {
            let x = Euc32::new(i, 13).unwrap();
            println!("{:02}: {}", i, x.stringify());
        }
    }

    #[test]
    fn equiv() {
        for interval in 1..=32 {
            for hits in 1..=interval {
                let mut ctr = 0u32;

                // TODO: There's probaby a *math* way to do this without
                // a loop, but whatever
                while ctr < interval {
                    ctr += hits;
                }

                let ctr_math = hits * (((interval - 1) / hits) + 1);
                assert_eq!(ctr, ctr_math);
            }
        }
    }
}

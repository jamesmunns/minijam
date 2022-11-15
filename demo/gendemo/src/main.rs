use std::{thread::sleep, time::Duration};

use thursday::phrdat::{PhraseDataParameters, PhraseDataBuilder};
use rand::thread_rng;

fn main() {
    let mut rng = thread_rng();
    let params = PhraseDataParameters::default();
    let mut builder = PhraseDataBuilder::default();
    println!("{:#?}", builder);

    for _ in 0..10 {
        builder.fill(&mut rng, &params);
        println!("{:#?}", builder);
        sleep(Duration::from_millis(2000));
    }

}

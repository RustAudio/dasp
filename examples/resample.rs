extern crate hound;
extern crate sample;

use hound::{WavReader, WavSpec, WavWriter};
use sample::interpolate::{Sinc, Converter};
use std::i16;

fn main() {
    let mut reader = WavReader::open("./assets/two_vowels.wav").unwrap();
    let samples: Vec<[f64; 1]> = reader.samples::<i32>().map(|sample| {
        [sample.unwrap() as f64 / i16::MAX as f64]
    }).collect();

    let sample_rate = reader.spec().sample_rate as f64;
    let new_sample_rate = 10000.0;
    let sinc = Sinc::new(50, samples.iter().cloned());
    let conv = Converter::scale_sample_hz(samples.iter().cloned(), sinc, new_sample_rate / sample_rate);

    let spec = WavSpec {
        channels: 1,
        sample_rate: new_sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create("./assets/two_vowels_10k.wav", spec).unwrap();
    for s in conv {
        writer.write_sample((s[0] * i16::MAX as f64) as i16).unwrap();
    }
}

extern crate find_folder;
extern crate hound;
extern crate sample;

use hound::{WavReader, WavSpec, WavWriter};
use sample::interpolate::{Sinc, Converter};
use sample::{signal, Sample, Signal};

fn main() {
    let assets = find_folder::Search::ParentsThenKids(5, 5).for_folder("assets").unwrap();
    let mut reader = WavReader::open(assets.join("two_vowels.wav")).unwrap();
    let samples: Vec<[f64; 1]> = reader.samples::<i16>()
        .map(|s| [s.unwrap().to_sample()])
        .collect();
    let len = samples.len();
    let signal = signal::from_slice(&samples[..]);

    let sample_rate = reader.spec().sample_rate as f64;
    let new_sample_rate = 10_000.0;
    let sinc = Sinc::zero_padded(50);
    let conv = Converter::from_hz_to_hz(signal, sinc, sample_rate, new_sample_rate);

    let spec = WavSpec {
        channels: 1,
        sample_rate: new_sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(assets.join("two_vowels_10k.wav"), spec).unwrap();
    let len = (len as f64 * new_sample_rate / sample_rate) as usize;
    for f in conv.take(len) {
        writer.write_sample((f[0].to_sample::<i16>())).unwrap();
    }
}

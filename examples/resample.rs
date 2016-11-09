extern crate find_folder;
extern crate hound;
extern crate sample;

use hound::{WavReader, WavSpec, WavWriter};
use sample::interpolate::{Sinc, Converter};
use sample::Sample;

fn main() {
    let assets = find_folder::Search::ParentsThenKids(5, 5).for_folder("assets").unwrap();
    let mut reader = WavReader::open(assets.join("two_vowels.wav")).unwrap();
    let samples: Vec<[f64; 1]> = reader.samples::<i16>()
        .map(|s| [s.unwrap().to_sample::<f64>()])
        .collect();

    let sample_rate = reader.spec().sample_rate as f64;
    let new_sample_rate = 10_000.0;
    // Zero-pad the `Sinc` interpolater.
    let sinc = Sinc::new(50, sample::signal::equilibrium::<[f64; 1]>());
    let conv = Converter::from_hz_to_hz(samples.iter().cloned(), sinc, sample_rate, new_sample_rate);

    let spec = WavSpec {
        channels: 1,
        sample_rate: new_sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(assets.join("two_vowels_10k.wav"), spec).unwrap();
    for f in conv {
        writer.write_sample((f[0].to_sample::<i16>())).unwrap();
    }
}

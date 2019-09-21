// An example of using `sample` to efficiently perform decent quality sample rate conversion on a
// WAV file entirely on the stack.

extern crate find_folder;
extern crate hound;
extern crate sample;

use hound::{WavReader, WavWriter};
#[cfg(all(feature = "interpolate", feature = "ring_buffer", feature = "signal"))]
use sample::{interpolate, ring_buffer, signal, Sample, Signal};

#[cfg(all(feature = "interpolate", feature = "ring_buffer", feature = "signal"))]
fn main() {
    // Find and load the wav.
    let assets = find_folder::Search::ParentsThenKids(5, 5).for_folder("assets").unwrap();
    let reader = WavReader::open(assets.join("two_vowels.wav")).unwrap();

    // Get the wav spec and create a target with the new desired sample rate.
    let spec = reader.spec();
    let mut target = spec;
    target.sample_rate = 10_000;

    // Read the interleaved samples and convert them to a signal.
    let samples = reader.into_samples().filter_map(Result::ok).map(i16::to_sample::<f64>);
    let signal = signal::from_interleaved_samples_iter(samples);

    // Convert the signal's sample rate using `Sinc` interpolation.
    let ring_buffer = ring_buffer::Fixed::from([[0.0]; 100]);
    let sinc = interpolate::Sinc::new(ring_buffer);
    let new_signal = signal.from_hz_to_hz(sinc, spec.sample_rate as f64, target.sample_rate as f64);

    // Write the result to a new file.
    let mut writer = WavWriter::create(assets.join("two_vowels_10k.wav"), target).unwrap();
    for frame in new_signal.until_exhausted() {
        writer.write_sample(frame[0].to_sample::<i16>()).unwrap();
    }
}

#[cfg(not(all(feature = "interpolate", feature = "ring_buffer", feature = "signal")))]
fn main() {
    panic!("This example only works when compiled with the features 'interpolate', 'ring_buffer' and 'signal'.");
}

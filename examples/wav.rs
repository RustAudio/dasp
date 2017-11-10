extern crate find_folder;
extern crate hound;
extern crate portaudio as pa;
extern crate sample;

use sample::{signal, Signal, ToFrameSliceMut};
use sample::interpolate::Linear;

// Thumb piano.
mod wav {
    pub const NUM_CHANNELS: usize = 2;
    pub const PATH: &'static str = "thumbpiano A#3.wav";
    pub type Frame = [i16; NUM_CHANNELS];
}

const FRAMES_PER_BUFFER: u32 = 64;
const SAMPLE_RATE: f64 = 44_100.0;


fn main() {
    run().unwrap();
}

fn run() -> Result<(), pa::Error> {
    // Get the frames to play back.
    let frames: Vec<wav::Frame> = frames(wav::PATH);
    let mut signal = frames.clone().into_iter();

    // Initialise PortAudio.
    let pa = try!(pa::PortAudio::new());
    let settings = try!(pa.default_output_stream_settings::<i16>(
        wav::NUM_CHANNELS as i32,
        SAMPLE_RATE,
        FRAMES_PER_BUFFER,
    ));

    // Define the callback which provides PortAudio the audio.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, .. }| {
        let buffer: &mut [wav::Frame] = buffer.to_frame_slice_mut().unwrap();
        for out_frame in buffer {
            match signal.next() {
                Some(frame) => *out_frame = frame,
                None => return pa::Complete,
            }
        }
        pa::Continue
    };

    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));
    try!(stream.start());

    while let Ok(true) = stream.is_active() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    try!(stream.stop());
    try!(stream.close());

    Ok(())
}

// Given the file name, produces a Vec of `Frame`s which may be played back.
fn frames(file_name: &'static str) -> Vec<wav::Frame> {
    let assets = find_folder::Search::ParentsThenKids(5, 5)
        .for_folder("assets")
        .unwrap();
    let sample_file = assets.join(file_name);
    let mut reader = hound::WavReader::open(&sample_file).unwrap();
    let spec = reader.spec();
    let duration = reader.duration();
    let new_duration = (duration as f64 * (SAMPLE_RATE as f64 / spec.sample_rate as f64)) as usize;
    let samples = reader.samples().map(|s| s.unwrap());
    let mut signal = signal::from_interleaved_samples_iter::<_, wav::Frame>(samples);
    let interp = Linear::from_source(&mut signal);
    signal
        .from_hz_to_hz(interp, spec.sample_rate as f64, SAMPLE_RATE as f64)
        .take(new_duration)
        .collect()
}

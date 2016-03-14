extern crate find_folder;
extern crate hound;
extern crate portaudio as pa;
extern crate sample;

use sample::{signal, Signal, ToFrameSliceMut};

const FRAMES_PER_BUFFER: u32 = 64;
const SAMPLE_RATE: f64 = 96_000.0;

// // Amen break.
// mod wav {
//     pub const NUM_CHANNELS: usize = 1;
//     pub const PATH: &'static str = "amen_break.wav";
//     pub type Frame = [i16; NUM_CHANNELS];
// }

// Thumb piano.
mod wav {
    pub const NUM_CHANNELS: usize = 2;
    pub const PATH: &'static str = "thumbpiano A#3.wav";
    pub type Frame = [i16; NUM_CHANNELS];
}


fn main() {
    run().unwrap();
}

// Given the file name, produces a Vec of frames which may be played back.
fn frames(file_name: &'static str) -> Vec<wav::Frame> {
    let assets = find_folder::Search::ParentsThenKids(5, 5).for_folder("assets").unwrap();
    let sample_file = assets.join(file_name);
    let mut reader = hound::WavReader::open(&sample_file).unwrap();
    let spec = reader.spec();
    let samples = reader.samples().map(|s| s.unwrap());
    let source_hz = spec.sample_rate as f64;
    let target_hz = SAMPLE_RATE as f64;
    signal::from_samples::<_, wav::Frame>(samples)
        .scale_amp(0.5)
        .from_hz_to_hz(source_hz, target_hz)
        .collect()
}

fn run() -> Result<(), pa::Error> {
    // Get the frames to play back.
    let frames: Vec<wav::Frame> = frames(wav::PATH);
    let mut signal = frames.clone().into_iter();

    // Initialise PortAudio.
    let pa = try!(pa::PortAudio::new());
    let settings = try!(pa.default_output_stream_settings::<i16>(wav::NUM_CHANNELS as i32,
                                                                 SAMPLE_RATE,
                                                                 FRAMES_PER_BUFFER));

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

    while let Ok(true) = stream.is_active() {}

    try!(stream.stop());
    try!(stream.close());

    Ok(())
}

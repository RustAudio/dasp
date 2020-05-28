use portaudio as pa;

use dasp::{signal, Frame, Sample, Signal};
use dasp::slice::ToFrameSliceMut;

const FRAMES_PER_BUFFER: u32 = 512;
const NUM_CHANNELS: i32 = 1;
const SAMPLE_RATE: f64 = 44_100.0;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), pa::Error> {
    // Create a signal chain to play back 1 second of each oscillator at A4.
    let hz = signal::rate(SAMPLE_RATE).const_hz(440.0);
    let one_sec = SAMPLE_RATE as usize;
    let mut waves = hz.clone()
        .sine()
        .take(one_sec)
        .chain(hz.clone().saw().take(one_sec))
        .chain(hz.clone().square().take(one_sec))
        .chain(hz.clone().noise_simplex().take(one_sec))
        .chain(signal::noise(0).take(one_sec))
        .map(|f| f.map(|s| s.to_sample::<f32>() * 0.2));

    // Initialise PortAudio.
    let pa = pa::PortAudio::new()?;
    let settings = pa.default_output_stream_settings::<f32>(
        NUM_CHANNELS,
        SAMPLE_RATE,
        FRAMES_PER_BUFFER,
    )?;

    // Define the callback which provides PortAudio the audio.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, .. }| {
        let buffer: &mut [[f32; 1]] = buffer.to_frame_slice_mut().unwrap();
        for out_frame in buffer {
            match waves.next() {
                Some(frame) => *out_frame = frame,
                None => return pa::Complete,
            }
        }
        pa::Continue
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;
    stream.start()?;

    while let Ok(true) = stream.is_active() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    stream.stop()?;
    stream.close()?;

    Ok(())
}

use dasp::signal::{self, Signal};
use dasp::slice::ToFrameSliceMut;
use portaudio as pa;

fn main() {
    // Find and load the wav.
    let assets = find_folder::Search::ParentsThenKids(5, 5).for_folder("assets").unwrap();
    let reader = hound::WavReader::open(assets.join("thumbpiano A#3.wav")).unwrap();
    let spec = reader.spec();

    // Read the interleaved samples and convert them to a signal.
    let samples = reader.into_samples::<i16>().filter_map(Result::ok);
    let mut frames = signal::from_interleaved_samples_iter(samples).until_exhausted();

    // Initialise PortAudio.
    let pa = pa::PortAudio::new().unwrap();
    let ch = spec.channels as i32;
    let sr = spec.sample_rate as f64;
    let buffer_len = 0; // use default
    let settings = pa.default_output_stream_settings::<i16>(ch, sr, buffer_len).unwrap();

    // A channel for indicating when playback has completed.
    let (complete_tx, complete_rx) = std::sync::mpsc::channel();

    // Define the callback which provides PortAudio the audio.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, .. }| {
        let buffer: &mut [[i16; 2]] = buffer.to_frame_slice_mut().unwrap();
        for out_frame in buffer {
            match frames.next() {
                Some(frame) => *out_frame = frame,
                None => {
                    complete_tx.send(()).unwrap();
                    return pa::Complete;
                },
            }
        }
        pa::Continue
    };

    // Spawn and start the output stream.
    let mut stream = pa.open_non_blocking_stream(settings, callback).unwrap();
    stream.start().unwrap();

    // Block until playback completes.
    complete_rx.recv().unwrap();

    stream.stop().unwrap();
    stream.close().unwrap();
}

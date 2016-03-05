use {Sample, Duplex};

/// Convert the `current_samples` from the given sample rate `current_rate` to `target_rate`.
///
/// The result is written to the `target_samples` buffer, which will be resized if necessary.
pub fn conv<S>(current_samples: &[S],
               current_n_frames: usize,
               current_rate: u32,
               target_samples: &mut Vec<S>,
               target_rate: u32)
    where S: Sample + Duplex<f32>,
{
    // Sample rates must be greater than 0.
    assert!(current_rate > 0.0 && target_rate > 0.0);

    // The number of channels and the number of frames must be a multiple of the number of samples.
    assert!(target_samples.len() % current_n_frames == 0);

    let n_channels = target_samples / current_n_frames;
    let rate_ratio = target_rate as f64 / current_rate as f64;
    let current_len = current_samples.len();
    let target_n_frames = current_n_frames as f64 * rate_ratio;
    let target_len = target_n_frames * n_channels;

    // Ensure that the `target_samples` buffer has the required capacity and is set to the
    // correct length.
    //
    // We use `set_len` so that we don't have to initialise every index with an arbitrary value as
    // we're about to write over them again anyway.
    //
    // We know that `set_len` is safe as we're guaranteed to fill every index with a valid sample.
    if target_samples.len() != target_len {
        target_samples.reserve(target_len);
        unsafe {
            target_samples.set_len(target_len);
        }
    }

    let step = 1.0 / target_n_frames as f64;
    let mut weight = 0.0f64;
    let mut idx = 0;
    for _ in 0..target_n_frames {
        let current_idx = weight.floor() as usize;
        let next_idx = current_idx + 1;
        let remainder = weight.fract();
        for _ in 0..n_channels {
            let current_sample = current_samples[current_idx].to_sample::<f64>();
            let next_sample = current_samples[next_idx].to_sample::<f64>();
            let quantize_diff = next_sample - current_sample;
            target_samples[i] = (current_sample + (quantize_diff as f64 * remainder) as f32).to_sample();
            idx += 1;
        }
        weight += step;
    }
}

use {Sample, Duplex};

/// An iterator that converts 
pub struct RateConverter<I> {
    src_frames: I,
    /// The ratio between the target and source sample rates.
    ///
    /// This value is equal to `target_sample_rate / source_sample_rate`.
    ratio: f32,
}



// /// Convert the `current_samples` from the given sample rate `current_rate` to `target_rate`.
// ///
// /// The result is written to the `target_samples` buffer, which will be resized if necessary.
// ///
// /// **Panics** if either sample rate is less than or equal to 0 or if `current_n_frames` is not a
// /// multiple of `current_samples.len()`.
// pub fn conv<S>(current_samples: &[S],
//                current_n_frames: usize,
//                current_rate: u32,
//                target_samples: &mut Vec<S>,
//                target_rate: u32)
//     where S: Sample + Duplex<f32>,
// {
//     // Sample rates must be greater than 0.
//     assert!(current_rate > 0 && target_rate > 0);
// 
//     // The number of channels and the number of frames must be a multiple of the number of samples.
//     assert!(target_samples.len() % current_n_frames == 0);
// 
//     let n_channels = current_samples.len() / current_n_frames;
//     let rate_ratio = target_rate as f64 / current_rate as f64;
//     let target_n_frames = (current_n_frames as f64 * rate_ratio) as usize;
//     let target_len = target_n_frames * n_channels;
// 
//     // Ensure that the `target_samples` buffer has the required capacity and is set to the
//     // correct length.
//     //
//     // We use `set_len` so that we don't have to initialise every index with an arbitrary value as
//     // we're about to write over them again anyway.
//     //
//     // We know that `set_len` is safe as we're guaranteed to fill every index with a valid sample.
//     if target_samples.len() != target_len {
//         target_samples.reserve(target_len);
//         unsafe {
//             target_samples.set_len(target_len);
//         }
//     }
// 
//     let step = (1.0 / target_n_frames as f64) / (1.0 / current_n_frames as f64);
//     let mut weight = 0.0f64;
//     let mut idx = 0;
//     for _ in 0..target_n_frames {
//         let current_frame = weight.floor() as usize;
//         let remainder = weight.fract();
//         //println!("idx: {:?}, weight: {:?}", idx, weight);
//         for channel in 0..n_channels {
//             let current_idx = (current_frame * n_channels) + channel;
//             let next_idx = current_idx + n_channels;
//             let current_sample = current_samples[current_idx].to_sample::<f32>();
//             // Careful to not index out of bounds on the current_sample buffer when getting the
//             // next sample for interpolation.
//             let next_sample = current_samples.get(next_idx).map(|s| s.to_sample::<f32>()).unwrap_or(current_sample);
//             let diff = next_sample - current_sample;
//             let amplitude = current_sample + (diff as f64 * remainder) as f32;
//             //println!("\tcurrent: {:?}, next: {:?}, diff: {:?}, amplitude: {:?}", current_sample, next_sample, diff, amplitude);
//             target_samples[idx] = amplitude.to_sample();
//             idx += 1;
//         }
//         weight += step;
//     }
// }

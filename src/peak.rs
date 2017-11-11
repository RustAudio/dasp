//! Peak envelope detection over a signal.

use {Frame, Sample};

/// A signal rectifier that produces only the positive samples.
pub fn positive_half_wave<F>(frame: F) -> F
where
    F: Frame,
{
    frame.map(|s| if s < Sample::equilibrium() {
        Sample::equilibrium()
    } else {
        s
    })
}

/// A signal rectifier that produces only the negative samples.
pub fn negative_half_wave<F>(frame: F) -> F
where
    F: Frame,
{
    frame.map(|s| if s > Sample::equilibrium() {
        Sample::equilibrium()
    } else {
        s
    })
}

/// A signal rectifier that produces the absolute amplitude from samples.
pub fn full_wave<F>(frame: F) -> F
where
    F: Frame,
{
    frame.map(|s| {
        let signed = s.to_signed_sample();
        if signed < Sample::equilibrium() {
            -signed
        } else {
            signed
        }.to_sample()
    })
}

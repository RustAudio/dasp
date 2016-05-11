use core::marker::PhantomData;
use core::f64;

use Sample;
use signal;
use signal::{Phase, ConstHz, Step};
use conv::{FromSample, ToSample};

// Using types instead of enum allows for static dispatch
pub trait WindowType<E: Step, S: Sample> {
    fn at_phase(phase: S) -> S;
}

pub struct HanningWindow;

impl<E: Step, S: Sample + ToSample<f64> + FromSample<f64>> WindowType<E, S> for HanningWindow {
    fn at_phase(phase: S) -> S {
        let v = phase.to_sample::<f64>() * f64::consts::PI * 2.;
        (0.5f64 * (1f64 - v.cos())).to_sample::<S>()
    }
}

pub struct Window<S: Sample, T: WindowType<ConstHz, S>> {
    pub phase: Phase<ConstHz>,
    wtype: PhantomData<T>,
    stype: PhantomData<S>
}

impl<S: Sample, T: WindowType<ConstHz, S>> Window<S, T> {
    pub fn new(len: usize) -> Window<S, T> {
        let step = signal::rate(len as f64).const_hz(1.);
        Window {
            phase: signal::phase(step),
            wtype: PhantomData,
            stype: PhantomData
        }
    }
}

impl<S: Sample + FromSample<f64>, T: WindowType<ConstHz, S>> Iterator for Window<S, T> {
    type Item = [S; 1];

    fn next(&mut self) -> Option<Self::Item> {
        Some([T::at_phase((self.phase.next_phase()).to_sample::<S>())])
    }
}


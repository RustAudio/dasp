use core::marker::PhantomData;
use core::f64;

use {Sample, FloatSample};
use signal;
use signal::{Phase, ConstHz, Step};
use conv::{FromSample, ToSample};
use frame::Frame;

// Using types instead of enum allows for static dispatch
pub trait WindowType<S: Sample, T: Step> {
    fn at_phase(phase: S) -> S;
}

pub struct HanningWindow;

impl<S: Sample + ToSample<f64> + FromSample<f64>, T: Step> WindowType<S, T> for HanningWindow {
    fn at_phase(phase: S) -> S {
        let v = phase.to_sample::<f64>() * f64::consts::PI * 2.;
        (0.5f64 * (1f64 - v.cos())).to_sample::<S>()
    }
}

pub struct RectangleWindow;

impl<S: Sample + FromSample<f64>, T: Step> WindowType<S, T> for RectangleWindow {
    #[allow(unused)]
    fn at_phase(phase: S) -> S {
        (1.).to_sample::<S>()
    }
}

pub struct Window<S: Sample, T: Step, F: Frame<Sample=S>, WT: WindowType<S, T>> {
    pub phase: Phase<ConstHz>,
    stype: PhantomData<S>,
    ttype: PhantomData<T>,
    ftype: PhantomData<F>,
    wttype: PhantomData<WT>
}

impl<S: Sample, F: Frame<Sample=S>, WT: WindowType<S, ConstHz>> Window<S, ConstHz, F, WT> {
    pub fn new(len: usize) -> Self {
        let step = signal::rate(len as f64).const_hz(1.);
        Window {
            phase: signal::phase(step),
            stype: PhantomData,
            ttype: PhantomData,
            ftype: PhantomData,
            wttype: PhantomData
        }
    }
}

impl<S: Sample + FromSample<f64>, F: Frame<Sample=S>, T: Step, WT: WindowType<S, T>> Iterator for Window<S, T, F, WT> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        let v = WT::at_phase((self.phase.next_phase()).to_sample::<S>());
        Some(F::from_fn(|_| v))
    }
}

pub struct WindowedFrame<'a, S: Sample + FromSample<f64>, T: Step, F: 'a + Frame<Sample=S>, WT: WindowType<S, T>> {
    pub data: &'a [F],
    pub window: Window<S, T, F, WT>,
    idx: usize
}

impl<'a, S: Sample + FromSample<f64>, T: Step, F: 'a + Frame<Sample=S>, WT: WindowType<S, T>> WindowedFrame<'a, S, T, F, WT> {
    pub fn new(data: &'a [F], window: Window<S, T, F, WT>) -> Self {
        WindowedFrame {
            data: data,
            window: window,
            idx: 0
        }
    }
}

impl<'a, S: Sample<Float=S> + FloatSample + FromSample<f64>, T: Step, F: 'a + Frame<Sample=S::Float>, WT: WindowType<S, T>> Iterator for WindowedFrame<'a, S, T, F, WT> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        self.window.next().and_then(|v| {
            let out = self.data.get(self.idx).and_then(|d| {
                Some(v.mul_amp(*d))
            });
            self.idx += 1;
            out
        })
    }
}

/// Windower takes a long slice of data and generates an iterator over its frames
pub struct Windower<'a, S: Sample + FromSample<f64>, T: Step, F: 'a + Frame<Sample=S>, WT: WindowType<S, T>> {
    pub bin: usize,
    pub hop: usize,
    pub idx: usize,
    pub data: &'a [F],
    stype: PhantomData<S>,
    ttype: PhantomData<T>,
    wttype: PhantomData<WT>
}

impl<'a, S: Sample + FromSample<f64>, T: Step, F: 'a + Frame<Sample=S>, WT: WindowType<S, T>> Windower<'a, S, T, F, WT> {
    /// Providing some reasonable defaults of bin = 512, hop = 256
    pub fn new(data: &'a [F]) -> Self {
        Windower {
            bin: 512,
            hop: 256,
            idx: 0,
            data: data,
            stype: PhantomData,
            ttype: PhantomData,
            wttype: PhantomData
        }
    }
}

impl<'a, S: Sample + FromSample<f64>, F: 'a + Frame<Sample=S>, WT: WindowType<S, ConstHz>> Iterator for Windower<'a, S, ConstHz, F, WT> {
    type Item = WindowedFrame<'a, S, ConstHz, F, WT>;

    fn next(&mut self) -> Option<Self::Item> {
        let top = self.idx + self.bin;
        if top < self.data.len() {
            let data = &self.data[self.idx..top];
            let window: Window<S, ConstHz, F, WT> = Window::new(self.bin);
            self.idx += self.hop;
            Some(WindowedFrame::new(data, window))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = (self.data.len() - self.idx) - (self.bin - self.hop);
        ((rem as f64 / self.hop as f64).ceil() as usize, None)
    }
}


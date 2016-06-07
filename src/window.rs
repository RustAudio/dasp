use core::marker::PhantomData;
use core::f64;

use {Sample, FloatSample};
use signal;
use signal::{Phase, ConstHz};
use conv::{FromSample, ToSample};
use frame::Frame;

// Using types instead of enum allows for static dispatch
pub trait Type<S: Sample> {
    fn at_phase(phase: S) -> S;
}

pub struct Hanning;

pub struct Rectangle;

pub struct Window<S, F, WT> 
    where S: Sample, 
          F: Frame<Sample=S>,
          WT: Type<S>
{
    pub phase: Phase<ConstHz>,
    stype: PhantomData<S>,
    ftype: PhantomData<F>,
    wttype: PhantomData<WT>
}

pub struct WindowedFrame<'a, S, F, WT> 
    where S: Sample + FromSample<f64>, 
          F: 'a + Frame<Sample=S>,
          WT: Type<S>
{
    pub data: &'a [F],
    pub window: Window<S, F, WT>,
    idx: usize
}

/// Windower takes a long slice of data and generates an iterator over its frames
pub struct Windower<'a, S, F, WT> 
    where S: Sample + FromSample<f64>, 
          F: 'a + Frame<Sample=S>, 
          WT: Type<S>
{
    pub bin: usize,
    pub hop: usize,
    pub idx: usize,
    pub data: &'a [F],
    stype: PhantomData<S>,
    wttype: PhantomData<WT>
}

impl<S: Sample + ToSample<f64> + FromSample<f64>> Type<S> for Hanning {
    fn at_phase(phase: S) -> S {
        let v = phase.to_sample::<f64>() * f64::consts::PI * 2.;
        (0.5f64 * (1f64 - v.cos())).to_sample::<S>()
    }
}

impl<S: Sample + FromSample<f64>> Type<S> for Rectangle {
    #[allow(unused)]
    fn at_phase(phase: S) -> S {
        (1.).to_sample::<S>()
    }
}

impl<S, F, WT> Window<S, F, WT> 
    where S: Sample, 
          F: Frame<Sample=S>,
          WT: Type<S>
{
    pub fn new(len: usize) -> Self {
        let step = signal::rate(len as f64 - 1.).const_hz(1.);
        Window {
            phase: signal::phase(step),
            stype: PhantomData,
            ftype: PhantomData,
            wttype: PhantomData
        }
    }
}

impl<S, F, WT> Iterator for Window<S, F, WT> 
    where S: Sample + FromSample<f64>,
          F: Frame<Sample=S>,
          WT: Type<S>
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        let v = WT::at_phase((self.phase.next_phase()).to_sample::<S>());
        Some(F::from_fn(|_| v))
    }
}

impl<'a, S: Sample + FromSample<f64>, F: 'a + Frame<Sample=S>, WT: Type<S>> WindowedFrame<'a, S, F, WT> {
    pub fn new(data: &'a [F], window: Window<S, F, WT>) -> Self {
        WindowedFrame {
            data: data,
            window: window,
            idx: 0
        }
    }
}

impl<'a, S, F, WT> Iterator for WindowedFrame<'a, S, F, WT> 
    where S: Sample<Float=S> + FloatSample + FromSample<f64>, 
          F: 'a + Frame<Sample=S::Float>, 
          WT: Type<S>
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        self.window.next().and_then(|v| {
            let out = self.data.get(self.idx)
                .and_then(|d| Some(v.mul_amp(*d)));
            self.idx += 1;
            out
        })
    }
}

impl<'a, S, F, WT> Windower<'a, S, F, WT> 
    where S: Sample + FromSample<f64>, 
          F: 'a + Frame<Sample=S>, 
          WT: Type<S>
{
    pub fn new(data: &'a [F], bin: usize, hop: usize) -> Self {
        Windower {
            bin: bin,
            hop: hop,
            idx: 0,
            data: data,
            stype: PhantomData,
            wttype: PhantomData
        }
    }
}

impl<'a, S, F, WT> Iterator for Windower<'a, S, F, WT> 
    where S: Sample + FromSample<f64>, 
          F: 'a + Frame<Sample=S>, 
          WT: Type<S>
{
    type Item = WindowedFrame<'a, S, F, WT>;

    fn next(&mut self) -> Option<Self::Item> {
        let top = self.idx + self.bin;
        if top < self.data.len() {
            let data = &self.data[self.idx..top];
            let window: Window<S, F, WT> = Window::new(self.bin);
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

pub fn hanning<S, F>(len: usize) -> Window<S, F, Hanning> 
    where S: Sample + FromSample<f64> + ToSample<f64>, 
          F: Frame<Sample=S>
{
    Window::new(len)
}

pub fn rectangle<S, F>(len: usize) -> Window<S, F, Rectangle> 
    where S: Sample + FromSample<f64>, 
          F: Frame<Sample=S>
{
    Window::new(len)
}


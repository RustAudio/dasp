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

pub struct Window<F, WT> 
    where F: Frame,
          WT: Type<F::Sample>
{
    pub phase: Phase<ConstHz>,
    ftype: PhantomData<F>,
    wttype: PhantomData<WT>
}

pub struct WindowedFrame<'a, F, WT> 
    where F: 'a + Frame,
          F::Sample: FromSample<f64>, 
          WT: Type<F::Sample>
{
    pub data: &'a [F],
    pub window: Window<F, WT>,
    idx: usize
}

/// Windower takes a long slice of data and generates an iterator over its frames
pub struct Windower<'a, F, WT> 
    where F: 'a + Frame, 
          F::Sample: FromSample<f64>, 
          WT: Type<F::Sample>
{
    pub bin: usize,
    pub hop: usize,
    pub idx: usize,
    pub data: &'a [F],
    wttype: PhantomData<WT>
}

impl<S: Sample + ToSample<f64> + FromSample<f64>> Type<S> for Hanning {
    fn at_phase(phase: S) -> S {
        let v = phase.to_sample::<f64>() * f64::consts::PI * 2.;
        (0.5f64 * (1f64 - super::cos(v))).to_sample::<S>()
    }
}

impl<S: Sample + FromSample<f64>> Type<S> for Rectangle {
    #[allow(unused)]
    fn at_phase(phase: S) -> S {
        (1.).to_sample::<S>()
    }
}

impl<F, WT> Window<F, WT> 
    where F: Frame,
          WT: Type<F::Sample>
{
    pub fn new(len: usize) -> Self {
        let step = signal::rate(len as f64 - 1.).const_hz(1.);
        Window {
            phase: signal::phase(step),
            ftype: PhantomData,
            wttype: PhantomData
        }
    }
}

impl<F, WT> Iterator for Window<F, WT> 
    where F: Frame, 
          F::Sample: FromSample<f64>,
          WT: Type<F::Sample>
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        let v = WT::at_phase((self.phase.next_phase()).to_sample::<F::Sample>());
        Some(F::from_fn(|_| v))
    }
}

impl<'a, F, WT> WindowedFrame<'a, F, WT> 
    where F: 'a + Frame, 
          F::Sample: FromSample<f64>, 
          WT: Type<F::Sample>
{
    pub fn new(data: &'a [F], window: Window<F, WT>) -> Self {
        WindowedFrame {
            data: data,
            window: window,
            idx: 0
        }
    }
}

impl<'a, F, WT> Iterator for WindowedFrame<'a, F, WT> 
    where F: 'a + Frame<Sample=<<F as Frame>::Sample as Sample>::Float>, 
          F::Sample: FloatSample + FromSample<f64>, 
          WT: Type<F::Sample>
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

impl<'a, F, WT> Windower<'a, F, WT> 
    where F: 'a + Frame, 
          F::Sample: FromSample<f64>, 
          WT: Type<F::Sample>
{
    pub fn new(data: &'a [F], bin: usize, hop: usize) -> Self {
        Windower {
            bin: bin,
            hop: hop,
            idx: 0,
            data: data,
            wttype: PhantomData
        }
    }
}

impl<'a, F, WT> Iterator for Windower<'a, F, WT> 
    where F: 'a + Frame, 
          F::Sample: FromSample<f64>, 
          WT: Type<F::Sample>
{
    type Item = WindowedFrame<'a, F, WT>;

    fn next(&mut self) -> Option<Self::Item> {
        let top = self.idx + self.bin;
        if top < self.data.len() {
            let data = &self.data[self.idx..top];
            let window: Window<F, WT> = Window::new(self.bin);
            self.idx += self.hop;
            Some(WindowedFrame::new(data, window))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = (self.data.len() - self.idx) - (self.bin - self.hop);
        (super::ceil(rem as f64 / self.hop as f64) as usize, None)
    }
}

pub fn hanning<F>(len: usize) -> Window<F, Hanning> 
    where F: Frame,
          F::Sample: FromSample<f64> + ToSample<f64> 
{
    Window::new(len)
}

pub fn rectangle<F>(len: usize) -> Window<F, Rectangle> 
    where F: Frame,
          F::Sample: FromSample<f64> 
{
    Window::new(len)
}


use {Amplitude, Duplex, Frame, Sample};
use std;


// Implement `Signal` for all `Iterator`s that yield `Frame`s.
impl<I> Signal for I
    where I: Iterator,
          I::Item: Frame, {}

/// An extension over `Iterator`s that yield `Frame`s.
pub trait Signal: Iterator + Sized
    where <Self as Iterator>::Item: Frame,
{

    /// Produces an `Iterator` that takes two `Signal`s, steps them forward in lockstep, sums each
    /// pair of `Frame`s together and yields the resulting `Frame`s.
    ///
    /// The `Iterator` will return `None` when either of the `Signal`s first yields `None`.
    #[inline]
    fn zip_add<S>(self, other: S) -> ZipAdd<Self, S>
        where S: Signal<Item=Self::Item>,
    {
        ZipAdd {
            signal_a: self,
            signal_b: other,
        }
    }

    /// Produces an `Iterator` that modulates the amplitude of `self` with `other` where `other` is
    /// some `Signal` yielding `Frame`s with `Sample`s that implement `Amplitude` (f32 or f64).
    ///
    /// The `Iterator` steps them forward in lockstep, multiplies the amplitude of each pair of
    /// `Frame`s together and yields the resulting `Frame`s.
    ///
    /// The `Iterator` will return `None` when either of the `Signal`s first yields `None`.
    #[inline]
    fn zip_mod_amp<S>(self, other: S) -> ZipModAmp<Self, S>
        where S: Signal,
              S::Item: Frame<NumChannels=<Self::Item as Frame>::NumChannels>,
              <Self::Item as Frame>::Sample: Duplex<<S::Item as Frame>::Sample>,
              <S::Item as Frame>::Sample: Amplitude,
    {
        ZipModAmp {
            signal_a: self,
            signal_b: other,
        }
    }

    /// Produces an `Iterator` that scales the amplitude of every `Frame` in `self` by the given
    /// `Amplitude` scalar.
    #[inline]
    fn scale_amp<A>(self, amp: A) -> ScaleAmp<Self, A>
        where <Self::Item as Frame>::Sample: Duplex<A>,
              A: Amplitude,
    {
        ScaleAmp {
            signal: self,
            amp: amp,
        }
    }

}


/// An iterator that endlessly yields `Frame`s of type `F` at equilibrium.
#[derive(Clone)]
pub struct Equilibrium<F> {
    frame_type: std::marker::PhantomData<F>,
}

/// An iterator that takes two `Signal`s, steps them forward in lockstep, sums each pair of
/// `Frame`s together and yields the resulting `Frame`s.
#[derive(Clone)]
pub struct ZipAdd<A, B> {
    signal_a: A,
    signal_b: B,
}

/// An `Iterator` that modulates the amplitude of `self` with `other`.
///
/// The `Iterator` steps them forward in lockstep, multiplies the amplitude of each pair of
/// `Frame`s together and yields the resulting `Frame`s.
///
/// The `Iterator` will return `None` when either of the `Signal`s first yields `None`.
#[derive(Clone)]
pub struct ZipModAmp<A, B> {
    signal_a: A,
    signal_b: B,
}

/// Produces an `Iterator` that scales the amplitude of every `Frame` in `self` by the given
/// `Amplitude` scalar.
#[derive(Clone)]
pub struct ScaleAmp<S, A> {
    signal: S,
    amp: A,
}


/// Provides an iterator that endlessly yields `Frame`s of type `F` at equilibrium.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::Signal;
///
/// fn main() {
///     let equilibrium: Vec<[f32; 1]> = sample::signal::equilibrium().take(4).collect();
///     assert_eq!(equilibrium, vec![[0.0], [0.0], [0.0], [0.0]]);
///
///     let equilibrium: Vec<[u8; 2]> = sample::signal::equilibrium().take(3).collect();
///     assert_eq!(equilibrium, vec![[128, 128], [128, 128], [128, 128]]);
/// }
/// ```
pub fn equilibrium<F>() -> Equilibrium<F> {
    Equilibrium { frame_type: std::marker::PhantomData }
}


impl<F> Iterator for Equilibrium<F>
    where F: Frame,
{
    type Item = F;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(F::equilibrium())
    }
}

impl<A, B> Iterator for ZipAdd<A, B>
    where A: Signal,
          B: Signal<Item=A::Item>,
          A::Item: Frame,
{
    type Item = A::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.signal_a.next().and_then(|f_a| self.signal_b.next().map(|f_b| f_a.add(f_b)))
    }
}

impl<A, B> Iterator for ZipModAmp<A, B>
    where A: Signal,
          B: Signal,
          A::Item: Frame,
          B::Item: Frame<NumChannels=<A::Item as Frame>::NumChannels>,
          <A::Item as Frame>::Sample: Duplex<<B::Item as Frame>::Sample>,
          <B::Item as Frame>::Sample: Amplitude,
{
    type Item = A::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.signal_a.next().and_then(|f_a| self.signal_b.next().map(|f_b| {
            f_a.zip_map(f_b, |s_a, s_b| s_a.scale_amplitude(s_b))
        }))
    }
}

impl<S, A> Iterator for ScaleAmp<S, A>
    where S: Signal,
          S::Item: Frame,
          <S::Item as Frame>::Sample: Duplex<A>,
          A: Amplitude,
{
    type Item = S::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.signal.next().map(|f| f.scale_amplitude(self.amp))
    }
}

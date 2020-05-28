#[cfg(not(feature = "std"))]
type Box<T> = alloc::boxed::Box<T>;
#[cfg(feature = "std")]
type Box<T> = std::boxed::Box<T>;

impl<S> Signal for Box<S>
where
    S: Signal + ?Sized,
{
    type Frame = S::Frame;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        (**self).next()
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        (**self).is_exhausted()
    }
}

//! Tests for the packed 24-bit sample types.
//!
//! The conversion matrix in `conv.rs` already covers the *values* these types produce. What is
//! tested here is everything that is specific to their being packed: their memory layout, their
//! byte order, and the traits whose derived implementations would have been wrong.

use dasp_sample::types::{i24_be3, i24_le3, u24_be3, u24_le3, I20, U20};
use dasp_sample::{Sample, SignedSample, I24, I24BE3, I24LE3, I48, U24, U24BE3, U24LE3, U48};
use std::mem;

/// The packed types offer the same `From` conversions and `Neg` as their padded equivalents.
///
/// These are value-widening conversions rather than amplitude-preserving ones — `I24::from(3i8)`
/// is `I24(3)`, not `I24(3 << 16)` — and the packed types match that exactly.
#[test]
fn from_and_neg_match_the_padded_types() {
    // Signed packed: the same list `I24` gets.
    assert_eq!(I24LE3::from(3i8).inner(), I24::from(3i8).inner());
    assert_eq!(I24LE3::from(3i16).inner(), I24::from(3i16).inner());
    assert_eq!(I24LE3::from(3i32).inner(), I24::from(3i32).inner());
    assert_eq!(I24LE3::from(3u8).inner(), I24::from(3u8).inner());
    assert_eq!(I24LE3::from(3u16).inner(), I24::from(3u16).inner());
    assert_eq!(I24LE3::from(I20::new_unchecked(3)).inner(), 3);
    assert_eq!(I24LE3::from(U20::new_unchecked(3)).inner(), 3);
    assert_eq!(I24BE3::from(3i16).inner(), 3);

    // Unsigned packed: the same list `U24` gets, which has no signed sources and no `Neg`.
    assert_eq!(U24LE3::from(3u8).inner(), U24::from(3u8).inner());
    assert_eq!(U24LE3::from(3u16).inner(), U24::from(3u16).inner());
    assert_eq!(U24BE3::from(3u16).inner(), 3);
    assert_eq!(U24LE3::from(U20::new_unchecked(3)).inner(), 3);

    // `From<i32>` wraps for both, which is the one place they agree *because* both wrap rather
    // than because neither overflows.
    assert_eq!(
        I24LE3::from(8_388_608i32).inner(),
        I24::from(8_388_608i32).inner()
    );

    assert_eq!((-I24LE3::new_unchecked(5)).inner(), -5);
    assert_eq!((-I24BE3::new_unchecked(5)).inner(), -5);
    // `MIN` has no positive counterpart in 24 bits, so negating it wraps back to itself.
    assert_eq!((-i24_le3::MIN).inner(), i24_le3::MIN.inner());

    // `I48`/`U48` already accepted `I24`/`U24`; they now accept the packed types too.
    assert_eq!(I48::from(I24LE3::new_unchecked(-7)).inner(), -7);
    assert_eq!(I48::from(I24BE3::new_unchecked(-7)).inner(), -7);
    assert_eq!(I48::from(U24LE3::new_unchecked(7)).inner(), 7);
    assert_eq!(U48::from(U24LE3::new_unchecked(7)).inner(), 7);
    assert_eq!(U48::from(U24BE3::new_unchecked(7)).inner(), 7);
}

/// The signed packed types are `SignedSample`, as every other signed sample type is, and the
/// arithmetic operators behave exactly as the padded ones do.
///
/// Between this, the `From` conversions and `Neg`, a call site can be retargeted from `I24` to
/// `I24LE3` by changing the type name — for everything except the bitwise and shift operators,
/// which the packed types deliberately do not have.
///
/// The release-mode wrap is free here: truncating to three bytes *is* reduction modulo 2^24, so
/// it lands on the same value the padded types reach through `wrap_overflow_once`.
#[test]
fn arithmetic_matches_the_padded_types() {
    fn assert_signed_sample<S: SignedSample>() {}
    assert_signed_sample::<I24LE3>();
    assert_signed_sample::<I24BE3>();
    // The unsigned pair is not, exactly as `U24` is not.

    // The same operations `tests/types.rs` checks on the padded types, with the same answers.
    macro_rules! check_arithmetic {
        ($T:ident) => {{
            let n = |v| $T::new_unchecked(v);
            assert_eq!((n(8) + n(12)).inner(), 20);
            assert_eq!((n(12) - n(4)).inner(), 8);
            assert_eq!((n(2) * n(2)).inner(), 4);
            assert_eq!((n(3) * n(3)).inner(), 9);
            assert_eq!((n(5) * n(10)).inner(), 50);
            assert_eq!((n(16) / n(8)).inner(), 2);
            assert_eq!((n(8) % n(3)).inner(), 2);
        }};
    }
    check_arithmetic!(I24LE3);
    check_arithmetic!(I24BE3);
    check_arithmetic!(U24LE3);
    check_arithmetic!(U24BE3);

    // Identical to the padded types for the same inputs.
    assert_eq!(
        (I24LE3::new_unchecked(8) + I24LE3::new_unchecked(12)).inner(),
        (I24::new_unchecked(8) + I24::new_unchecked(12)).inner()
    );
    assert_eq!(
        (I24LE3::new_unchecked(5) * I24LE3::new_unchecked(10)).inner(),
        (I24::new_unchecked(5) * I24::new_unchecked(10)).inner()
    );

    // The bitwise and shift operators are deliberately absent, so these do not compile:
    //   let _ = I24LE3::new_unchecked(1) & I24LE3::new_unchecked(1);
    //   let _ = !I24LE3::new_unchecked(1);
}

/// Overflow panics in debug, matching `I24`'s `Add`. In release it wraps instead, which is
/// checked in `overflow_wraps_in_release`.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "arithmetic operation overflowed")]
fn add_panics_on_overflow_in_debug() {
    let _ = i24_le3::MAX + I24LE3::new_unchecked(1);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "arithmetic operation overflowed")]
fn sub_panics_on_underflow_in_debug() {
    let _ = i24_le3::MIN - I24LE3::new_unchecked(1);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "arithmetic operation overflowed")]
fn mul_panics_on_overflow_in_debug() {
    let _ = i24_le3::MAX * I24LE3::new_unchecked(2);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "arithmetic operation overflowed")]
fn unsigned_sub_panics_on_underflow_in_debug() {
    let _ = u24_le3::MIN - U24LE3::new_unchecked(1);
}

/// In release the packed types wrap, landing on the same value the padded types reach through
/// `wrap_overflow_once`.
#[cfg(not(debug_assertions))]
#[test]
fn overflow_wraps_in_release() {
    assert_eq!(
        (i24_le3::MAX + I24LE3::new_unchecked(1)).inner(),
        -8_388_608
    );
    assert_eq!(
        (i24_le3::MAX + I24LE3::new_unchecked(1)).inner(),
        (dasp_sample::types::i24::MAX + I24::new_unchecked(1)).inner()
    );
}

/// The whole reason these types exist: a buffer of packed 24-bit PCM must be reinterpretable as a
/// slice of samples, which requires a stride of exactly three bytes and no alignment requirement.
#[test]
fn layout() {
    assert_eq!(mem::size_of::<I24LE3>(), 3);
    assert_eq!(mem::size_of::<I24BE3>(), 3);
    assert_eq!(mem::size_of::<U24LE3>(), 3);
    assert_eq!(mem::size_of::<U24BE3>(), 3);

    assert_eq!(mem::align_of::<I24LE3>(), 1);
    assert_eq!(mem::align_of::<I24BE3>(), 1);
    assert_eq!(mem::align_of::<U24LE3>(), 1);
    assert_eq!(mem::align_of::<U24BE3>(), 1);
}

/// The byte order is a property of the type, not of the host, so these assertions hold on a
/// big-endian target too.
#[test]
fn byte_order() {
    let n = 0x123456;
    assert_eq!(I24LE3::new_unchecked(n).to_bytes(), [0x56, 0x34, 0x12]);
    assert_eq!(I24BE3::new_unchecked(n).to_bytes(), [0x12, 0x34, 0x56]);
    assert_eq!(U24LE3::new_unchecked(n).to_bytes(), [0x56, 0x34, 0x12]);
    assert_eq!(U24BE3::new_unchecked(n).to_bytes(), [0x12, 0x34, 0x56]);

    assert_eq!(I24LE3::from_bytes([0x56, 0x34, 0x12]).inner(), 0x123456);
    assert_eq!(I24BE3::from_bytes([0x12, 0x34, 0x56]).inner(), 0x123456);
    assert_eq!(U24LE3::from_bytes([0x56, 0x34, 0x12]).inner(), 0x123456);
    assert_eq!(U24BE3::from_bytes([0x12, 0x34, 0x56]).inner(), 0x123456);
}

/// The signed types must sign extend from bit 23; the unsigned types must not.
#[test]
fn sign_extension() {
    assert_eq!(I24LE3::from_bytes([0x00, 0x00, 0x80]).inner(), -8_388_608);
    assert_eq!(I24BE3::from_bytes([0x80, 0x00, 0x00]).inner(), -8_388_608);
    assert_eq!(I24LE3::from_bytes([0xFF, 0xFF, 0xFF]).inner(), -1);
    assert_eq!(I24BE3::from_bytes([0xFF, 0xFF, 0xFF]).inner(), -1);

    assert_eq!(U24LE3::from_bytes([0x00, 0x00, 0x80]).inner(), 8_388_608);
    assert_eq!(U24BE3::from_bytes([0x80, 0x00, 0x00]).inner(), 8_388_608);
    assert_eq!(U24LE3::from_bytes([0xFF, 0xFF, 0xFF]).inner(), 16_777_215);
    assert_eq!(U24BE3::from_bytes([0xFF, 0xFF, 0xFF]).inner(), 16_777_215);
}

#[test]
fn consts() {
    assert_eq!(i24_le3::MIN.inner(), -8_388_608);
    assert_eq!(i24_le3::MAX.inner(), 8_388_607);
    assert_eq!(i24_le3::EQUILIBRIUM.inner(), 0);
    assert_eq!(i24_be3::MIN.inner(), -8_388_608);
    assert_eq!(i24_be3::MAX.inner(), 8_388_607);
    assert_eq!(i24_be3::EQUILIBRIUM.inner(), 0);

    assert_eq!(u24_le3::MIN.inner(), 0);
    assert_eq!(u24_le3::MAX.inner(), 16_777_215);
    assert_eq!(u24_le3::EQUILIBRIUM.inner(), 8_388_608);
    assert_eq!(u24_be3::MIN.inner(), 0);
    assert_eq!(u24_be3::MAX.inner(), 16_777_215);
    assert_eq!(u24_be3::EQUILIBRIUM.inner(), 8_388_608);

    // The `Sample::EQUILIBRIUM` associated consts must agree with the module consts.
    assert_eq!(<I24LE3 as Sample>::EQUILIBRIUM, i24_le3::EQUILIBRIUM);
    assert_eq!(<I24BE3 as Sample>::EQUILIBRIUM, i24_be3::EQUILIBRIUM);
    assert_eq!(<U24LE3 as Sample>::EQUILIBRIUM, u24_le3::EQUILIBRIUM);
    assert_eq!(<U24BE3 as Sample>::EQUILIBRIUM, u24_be3::EQUILIBRIUM);

    assert_eq!(I24LE3::from(I24::EQUILIBRIUM), i24_le3::EQUILIBRIUM);
    assert_eq!(U24BE3::from(U24::EQUILIBRIUM), u24_be3::EQUILIBRIUM);
}

#[test]
fn new_checks_range() {
    assert!(I24LE3::new(-8_388_609).is_none());
    assert!(I24LE3::new(8_388_608).is_none());
    assert_eq!(I24LE3::new(-8_388_608), Some(i24_le3::MIN));
    assert_eq!(I24LE3::new(8_388_607), Some(i24_le3::MAX));

    assert!(U24BE3::new(-1).is_none());
    assert!(U24BE3::new(16_777_216).is_none());
    assert_eq!(U24BE3::new(0), Some(u24_be3::MIN));
    assert_eq!(U24BE3::new(16_777_215), Some(u24_be3::MAX));
}

/// A packed type has nowhere to put an out-of-range value, so `new_unchecked` wraps modulo 2^24.
/// This is the one place where a packed type does not agree with its padded equivalent, because
/// the padded one can park an out-of-range value in the spare byte of its container.
#[test]
fn out_of_range_wraps() {
    assert_eq!(I24LE3::new_unchecked(8_388_608).inner(), -8_388_608);
    assert_eq!(I24LE3::new_unchecked(-8_388_609).inner(), 8_388_607);
    assert_eq!(I24BE3::new_unchecked(8_388_608).inner(), -8_388_608);
    assert_eq!(U24LE3::new_unchecked(16_777_216).inner(), 0);
    assert_eq!(U24BE3::new_unchecked(-1).inner(), 16_777_215);

    // The padded `From<i32>` wraps in exactly the same way...
    assert_eq!(I24::from(8_388_608i32).inner(), -8_388_608);
    // ...but the padded `new_unchecked` keeps the out-of-range value verbatim.
    assert_eq!(I24::new_unchecked(8_388_608).inner(), 8_388_608);

    // Which is why converting a float from outside the documented -1.0 <= v < 1.0 range gives
    // different answers for the padded and packed types.
    assert_eq!(1.0f32.to_sample::<I24>().inner(), 8_388_608);
    assert_eq!(1.0f32.to_sample::<I24LE3>().inner(), -8_388_608);
    assert_eq!(1.0f32.to_sample::<U24>().inner(), 16_777_216);
    assert_eq!(1.0f32.to_sample::<U24LE3>().inner(), 0);
}

/// Asserts `$lo < $hi` numerically *and* that a derived `Ord` would have got it wrong.
///
/// The second half is the point. Most orderings one might reach for happen to come out the same
/// either way, so an assertion like `MIN < MAX` passes under the very implementation it is
/// supposed to rule out. Requiring the byte-wise comparison to disagree makes each case prove it
/// is actually discriminating.
macro_rules! assert_derived_ord_would_fail {
    ($lo:expr, $hi:expr) => {{
        assert!($lo < $hi, "numeric ordering is wrong");
        assert!(
            $lo.to_bytes() > $hi.to_bytes(),
            "this case is not discriminating: byte order agrees with numeric order here, so it \
             would pass under a derived Ord and proves nothing"
        );
    }};
}

/// A derived `Ord` compares `[u8; 3]` lexicographically, which is wrong for three of the four
/// types: little-endian orders the *least* significant byte first, and the signed types read
/// their sign byte as unsigned.
#[test]
fn ordering_is_numeric_not_lexicographic() {
    // Signed, either byte order: the sign byte read as unsigned puts negatives above positives.
    assert_derived_ord_would_fail!(i24_le3::MIN, i24_le3::EQUILIBRIUM);
    assert_derived_ord_would_fail!(i24_be3::MIN, i24_be3::EQUILIBRIUM);
    assert_derived_ord_would_fail!(I24LE3::new_unchecked(-1), I24LE3::new_unchecked(1));
    assert_derived_ord_would_fail!(I24BE3::new_unchecked(-1), I24BE3::new_unchecked(1));

    // Little-endian, either signedness: 1 is [01,00,00] and 256 is [00,01,00], so the smaller
    // value sorts higher byte-wise even though both are positive.
    assert_derived_ord_would_fail!(I24LE3::new_unchecked(1), I24LE3::new_unchecked(256));
    assert_derived_ord_would_fail!(U24LE3::new_unchecked(1), U24LE3::new_unchecked(256));

    // `U24BE3` can have no discriminating case: for a big-endian *unsigned* value,
    // lexicographic byte order and numeric order are the same relation. Its manual `Ord` is
    // kept for consistency, not because a test could catch its absence.
    assert!(u24_be3::MIN < u24_be3::EQUILIBRIUM);
    assert!(u24_be3::EQUILIBRIUM < u24_be3::MAX);
    assert!(U24BE3::new_unchecked(1) < U24BE3::new_unchecked(256));

    // Ordering must agree with the derived `PartialEq`, which does compare bytes.
    let a = I24LE3::new_unchecked(42);
    assert_eq!(a.cmp(&a), std::cmp::Ordering::Equal);
    assert_eq!(a, I24LE3::new_unchecked(42));
}

/// Packing and unpacking is a pure re-arrangement of bits and so must be lossless for every value
/// in range, in both directions.
#[test]
fn round_trip_every_value() {
    for v in -8_388_608..=8_388_607i32 {
        assert_eq!(I24LE3::new_unchecked(v).inner(), v);
        assert_eq!(I24BE3::new_unchecked(v).inner(), v);
        assert_eq!(I24::from(I24LE3::new_unchecked(v)).inner(), v);
        assert_eq!(I24::from(I24BE3::new_unchecked(v)).inner(), v);
    }
    for v in 0..=16_777_215i32 {
        assert_eq!(U24LE3::new_unchecked(v).inner(), v);
        assert_eq!(U24BE3::new_unchecked(v).inner(), v);
        assert_eq!(U24::from(U24LE3::new_unchecked(v)).inner(), v);
        assert_eq!(U24::from(U24BE3::new_unchecked(v)).inner(), v);
    }
}

/// Every one of the 2^24 byte patterns must decode to a distinct value and re-encode to itself,
/// which is what makes the byte-wise `PartialEq` consistent with the value-wise `Ord`.
#[test]
fn every_byte_pattern_survives_a_round_trip() {
    for raw in 0..=0xFF_FFFFu32 {
        let bytes = [raw as u8, (raw >> 8) as u8, (raw >> 16) as u8];
        assert_eq!(
            I24LE3::new_unchecked(I24LE3::from_bytes(bytes).inner()).to_bytes(),
            bytes
        );
        assert_eq!(
            I24BE3::new_unchecked(I24BE3::from_bytes(bytes).inner()).to_bytes(),
            bytes
        );
        assert_eq!(
            U24LE3::new_unchecked(U24LE3::from_bytes(bytes).inner()).to_bytes(),
            bytes
        );
        assert_eq!(
            U24BE3::new_unchecked(U24BE3::from_bytes(bytes).inner()).to_bytes(),
            bytes
        );
    }
}

/// `add_amp` and `mul_amp` go via `Sample::Signed` and `Sample::Float`, so they must give the same
/// answers as the padded types.
#[test]
fn sample_ops() {
    let half = I24LE3::from_sample(0.5f32);
    assert_eq!(half.inner(), 4_194_304);
    assert_eq!(half.mul_amp(0.5).inner(), 2_097_152);
    assert_eq!(half.mul_amp(0.0), i24_le3::EQUILIBRIUM);
    assert_eq!(half.add_amp(I24LE3::new_unchecked(-4_194_304)).inner(), 0);

    // The unsigned pair takes its `Signed` from `U24`, which is `i32`, so the amplitude offset
    // is on the `i32` scale rather than the 24-bit one. The signed pair is its own `Signed`.
    let half = U24BE3::from_sample(0.5f32);
    assert_eq!(half.inner(), 12_582_912);
    assert_eq!(half.mul_amp(0.0), u24_be3::EQUILIBRIUM);
    assert_eq!(half.to_signed_sample(), 1_073_741_824i32);
    assert_eq!(half.add_amp(-1_073_741_824), u24_be3::EQUILIBRIUM);

    assert_eq!(
        I24LE3::from_sample(-1.0f32).to_signed_sample(),
        i24_le3::MIN
    );
    assert_eq!(I24LE3::from_sample(-1.0f32).to_float_sample(), -1.0f32);
    assert_eq!(U24LE3::from_sample(-1.0f32).to_float_sample(), -1.0f32);
}

/// Pins `Sample::Signed` and `Sample::Float` for all four types.
///
/// Without this, two of the four were unasserted: changing `I24BE3`'s `Signed` to `i32`
/// compiled and passed the entire workspace suite, despite making `add_amp` read its argument
/// on a scale 256x off. The associated types are only observable through the scale `add_amp`
/// works on, so that is what is checked here — for every type, not just one.
#[test]
fn signed_and_float_associated_types() {
    fn assert_signed<S: Sample<Signed = Expected>, Expected>() {}
    fn assert_float<S: Sample<Float = Expected>, Expected>() {}

    assert_signed::<I24LE3, I24LE3>();
    assert_signed::<I24BE3, I24BE3>();
    assert_signed::<U24LE3, i32>();
    assert_signed::<U24BE3, i32>();

    assert_float::<I24LE3, f32>();
    assert_float::<I24BE3, f32>();
    assert_float::<U24LE3, f32>();
    assert_float::<U24BE3, f32>();

    // Half scale offset back to equilibrium, in the units of that type's `Signed` -- the
    // scale is exactly what a wrong associated type changes.
    assert_eq!(
        I24LE3::from_sample(0.5f32).add_amp(I24LE3::new_unchecked(-4_194_304)),
        i24_le3::EQUILIBRIUM
    );
    assert_eq!(
        I24BE3::from_sample(0.5f32).add_amp(I24BE3::new_unchecked(-4_194_304)),
        i24_be3::EQUILIBRIUM
    );
    assert_eq!(
        U24LE3::from_sample(0.5f32).add_amp(-1_073_741_824i32),
        u24_le3::EQUILIBRIUM
    );
    assert_eq!(
        U24BE3::from_sample(0.5f32).add_amp(-1_073_741_824i32),
        u24_be3::EQUILIBRIUM
    );

    assert_eq!(I24LE3::from_sample(0.5f32).mul_amp(0.5).inner(), 2_097_152);
    assert_eq!(I24BE3::from_sample(0.5f32).mul_amp(0.5).inner(), 2_097_152);
    assert_eq!(
        U24LE3::from_sample(0.5f32).mul_amp(0.0),
        u24_le3::EQUILIBRIUM
    );
    assert_eq!(
        U24BE3::from_sample(0.5f32).mul_amp(0.0),
        u24_be3::EQUILIBRIUM
    );
}

/// The payoff: a byte buffer straight off the wire, reinterpreted without a copy.
#[test]
fn reinterpret_byte_buffer() {
    // Three S24_3LE samples: min, equilibrium, max.
    let raw: [u8; 9] = [
        0x00, 0x00, 0x80, // -8_388_608
        0x00, 0x00, 0x00, // 0
        0xFF, 0xFF, 0x7F, // 8_388_607
    ];

    // Sound because `I24LE3` is `repr(transparent)` over `[u8; 3]`: every byte pattern is a valid
    // value, the alignment is 1, and the length is an exact multiple of the stride.
    assert_eq!(raw.len() % mem::size_of::<I24LE3>(), 0);
    let samples: &[I24LE3] = unsafe {
        std::slice::from_raw_parts(
            raw.as_ptr() as *const I24LE3,
            raw.len() / mem::size_of::<I24LE3>(),
        )
    };

    assert_eq!(samples.len(), 3);
    assert_eq!(samples[0], i24_le3::MIN);
    assert_eq!(samples[1], i24_le3::EQUILIBRIUM);
    assert_eq!(samples[2], i24_le3::MAX);

    let floats: Vec<f32> = samples.iter().map(|s| s.to_sample::<f32>()).collect();
    assert_eq!(floats[0], -1.0);
    assert_eq!(floats[1], 0.0);
    assert!(floats[2] > 0.999_999 && floats[2] < 1.0);
}

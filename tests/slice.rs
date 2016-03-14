extern crate sample;

#[test]
fn test_add_slice() {
    let mut a = [[-0.5]; 32];
    let b = [[0.5]; 32];
    sample::slice::add_in_place(&mut a, &b);
    assert_eq!([[0.0]; 32], a);
}

#[test]
#[should_panic]
fn test_add_slice_panic() {
    let mut a = [[0.5]; 31];
    let b = [[0.5]; 32];
    sample::slice::add_in_place(&mut a, &b);
}

#[test]
fn test_write_slice() {
    let mut a = [[0.0]; 32];
    let b = [[1.0]; 32];
    sample::slice::write(&mut a, &b);
    assert_eq!([[1.0]; 32], a);
}

#[test]
#[should_panic]
fn test_write_slice_panic() {
    let mut a = [[0.0]; 31];
    let b = [[1.0]; 32];
    sample::slice::write(&mut a, &b);
}

#[test]
fn test_add_slice_with_amp_per_channel() {
    let mut a = [[0.5]; 32];
    let b = [[1.0]; 32];
    let amp = [0.5];
    sample::slice::add_in_place_with_amp_per_channel(&mut a, &b, amp);
    assert_eq!([[1.0]; 32], a);
}

#[test]
#[should_panic]
fn test_add_slice_with_amp_per_channel_panic() {
    let mut a = [[0.5]; 31];
    let b = [[1.0]; 32];
    let amp = [0.5];
    sample::slice::add_in_place_with_amp_per_channel(&mut a, &b, amp);
}

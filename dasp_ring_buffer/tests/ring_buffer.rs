use dasp_ring_buffer as ring_buffer;

#[test]
fn test_bounded_boxed_slice() {
    let mut rb = ring_buffer::Bounded::from(vec![0; 3].into_boxed_slice());
    assert_eq!(rb.push(1), None);
    assert_eq!(rb.push(2), None);
    assert_eq!(rb.push(3), None);
    assert_eq!(rb.push(4), Some(1));
}

#[test]
fn test_bounded_array() {
    let mut rb = ring_buffer::Bounded::from([0i32; 3]);
    assert_eq!(rb.push(1), None);
    assert_eq!(rb.push(2), None);
    assert_eq!(rb.push(3), None);
    assert_eq!(rb.push(4), Some(1));
}

#[test]
#[should_panic]
fn text_bounded_from_empty_vec() {
    ring_buffer::Bounded::from(Vec::<i32>::new());
}

#[test]
fn test_bounded_from_vec() {
    let mut rb = ring_buffer::Bounded::from(vec![1, 2, 3]);
    assert_eq!(rb.push(4), None);
    assert_eq!(rb.push(5), None);
    assert_eq!(rb.push(6), None);
    assert_eq!(rb.push(7), Some(4));
}

#[test]
#[should_panic]
fn test_bounded_get_out_of_range() {
    let rb = ring_buffer::Bounded::from([0i32; 3]);
    let _ = rb[0];
}

#[test]
fn test_extend() {
    // Test each branch to look for UB.
    // space at end
    let mut rb = ring_buffer::Bounded::from([0i32; 4]);
    rb.push(1);
    rb.extend(&[2, 3][..]);
    assert_eq!(rb.slices(), (&[1, 2, 3][..], &[][..]));

    // old data wraps
    let mut rb = ring_buffer::Bounded::from([0i32; 4]);
    rb.extend(&[1, 2, 3, 4][..]);
    rb.pop();
    rb.pop();
    rb.pop();
    rb.push(5);
    rb.extend(&[6, 7][..]);
    assert_eq!(rb.slices(), (&[4][..], &[5, 6, 7][..]));

    // we wrap
    let mut rb = ring_buffer::Bounded::from([0i32; 4]);
    rb.extend(&[1, 2, 3][..]);
    rb.pop();
    rb.extend(&[4, 5][..]);
    assert_eq!(rb.slices(), (&[2, 3, 4][..], &[5][..]));
}

#[test]
#[should_panic]
fn test_extend_too_big() {
    let mut rb = ring_buffer::Bounded::from([0i32; 3]);
    rb.extend(&[1, 2][..]);
    let other = [0i32; 3];
    rb.extend(&other[..])
}

#[test]
fn test_read() {
    // Test each branch to look for UB.
    // contiguous data
    let mut rb = ring_buffer::Bounded::from([0i32; 4]);
    rb.extend(&[1, 2, 3][..]);
    let mut other = [0i32; 2];
    rb.read(&mut other[..]);
    assert_eq!(rb.slices(), (&[3][..], &[][..]));
    assert_eq!(other, [1, 2]);

    // not contiguous data, we only draw from first half
    let mut rb = ring_buffer::Bounded::from([0i32; 4]);
    rb.extend(&[1, 2, 3, 4][..]);
    rb.pop();
    rb.pop();
    rb.push(5);
    let mut other = [0i32; 1];
    rb.read(&mut other[..]);
    assert_eq!(rb.slices(), (&[4][..], &[5][..]));
    assert_eq!(other, [3]);

    // not contiguous data, we only draw from both halves
    let mut rb = ring_buffer::Bounded::from([0i32; 4]);
    rb.extend(&[1, 2, 3, 4][..]);
    rb.pop();
    rb.pop();
    rb.push(5);
    let mut other = [0i32; 3];
    rb.read(&mut other[..]);
    assert_eq!(rb.slices(), (&[][..], &[][..]));
    assert_eq!(other, [3, 4, 5]);
}

#[test]
#[should_panic]
fn test_read_too_small() {
    let mut rb = ring_buffer::Bounded::from([0i32; 3]);
    rb.extend(&[1, 2][..]);
    let mut other = [0i32; 3];
    rb.read(&mut other[..])
}

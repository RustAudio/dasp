use dasp_ring_buffer as ring_buffer;

#[test]
fn test_bounded_boxed_slice() {
    let mut rb = ring_buffer::Bounded::boxed_slice(3);
    assert_eq!(rb.push(1), None);
    assert_eq!(rb.push(2), None);
    assert_eq!(rb.push(3), None);
    assert_eq!(rb.push(4), Some(1));
}

#[test]
fn test_bounded_array() {
    let mut rb = ring_buffer::Bounded::<[i32; 3]>::array();
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
    let rb = ring_buffer::Bounded::<[i32; 3]>::array();
    let _ = rb[0];
}

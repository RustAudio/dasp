extern crate sample;

#[test]
fn test() {
    let foo = [[0.0], [1.0], [0.0], [-1.0]];
    let mut bar = vec![];
    sample::rate::conv(&foo, 1, &mut bar, 2);
    assert_eq!(&bar[..], &[[0.0], [0.5], [1.0], [0.5], [0.0], [-0.5], [-1.0], [-1.0]][..]);
}

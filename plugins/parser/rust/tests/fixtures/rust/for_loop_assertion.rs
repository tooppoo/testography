use crate::calculator::add;

#[test]
fn test_add_in_loop() {
    for i in 0..3 {
        assert_eq!(add(i, 1), i + 1);
    }
}

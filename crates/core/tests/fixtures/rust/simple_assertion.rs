use crate::calculator::add;

#[test]
fn test_add_returns_sum() {
    assert_eq!(add(1, 2), 3);
}

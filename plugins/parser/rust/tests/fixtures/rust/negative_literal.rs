use crate::calculator::negate;

#[test]
fn test_negate_positive() {
    assert_eq!(negate(1), -1);
}

#[test]
fn test_negate_float() {
    assert_eq!(negate(1.0), -1.0);
}

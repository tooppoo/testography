use crate::math::add;
use crate::math::subtract;

#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
}

#[test]
fn test_subtract() {
    assert_eq!(subtract(5, 3), 2);
}

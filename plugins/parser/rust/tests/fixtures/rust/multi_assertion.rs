use crate::validator::is_valid;

#[test]
fn test_valid_inputs() {
    assert!(is_valid("hello"));
    assert_eq!(is_valid("world"), true);
    assert_ne!(is_valid(""), false);
}

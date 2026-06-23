use crate::calculator;

#[test]
fn test_via_module_prefix() {
    assert_eq!(calculator::add(1, 2), 3);
}

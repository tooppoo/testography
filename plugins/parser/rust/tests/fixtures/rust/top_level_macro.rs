generate_tests!(calculator::add);

#[test]
fn explicit_test() {
    assert_eq!(1 + 1, 2);
}

use crate::values::{identity, number as renamed_number};

#[test]
fn literal_classification() {
    assert_eq!(renamed_number(0, 1.5, false), false);
    assert_eq!(identity("   "), "");
}

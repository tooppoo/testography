use crate::items::empty_items;

#[test]
fn empty_items_are_empty() {
    assert_eq!(empty_items(), []);
}

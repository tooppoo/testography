#[cfg(test)]
mod tests {
    use crate::calculator::add;

    #[test]
    fn nested_test_add() {
        let value = add(1, 2);
        assert_eq!(add(1, 2), 3);
        assert!(value > 0);
    }
}

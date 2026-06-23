use crate::helpers::helper;

mod tests {
    #[test]
    fn nested_call_without_local_import() {
        assert_eq!(helper(1), 1);
    }
}

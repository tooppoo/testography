mod first {
    mod tests {
        #[test]
        fn first_uses_super_helper() {
            assert_eq!(super::helper(1), 1);
        }
    }
}

mod second {
    mod tests {
        #[test]
        fn second_uses_super_helper() {
            assert_eq!(super::helper(2), 2);
        }
    }
}

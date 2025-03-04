#[cfg(test)]
mod teest_layer_props {
    use crate::Inheritable;

    #[test]
    fn join_with_child() {
        let i = Inheritable::Custom(5);
        let i2 = Inheritable::Inherit;

        assert_eq!(i.join_with_child(&i2).unwrap(), 5);
    }
}

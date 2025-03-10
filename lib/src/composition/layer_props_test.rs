#[cfg(test)]
mod test_layer_props {
    use crate::Inheritable;

    #[test]
    fn overwrite_with() {
        let i = Inheritable::Specified(5);
        let i2 = Inheritable::Inherit;

        assert_eq!(i.overwrite_with(&i2).unwrap(), 5);
    }
}

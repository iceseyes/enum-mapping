fn main() {}

#[cfg(test)]
mod tests {
    use enum_mapping_macro::U8Mapped;

    #[allow(dead_code)]
    #[derive(Debug, U8Mapped)]
    #[repr(u8)]
    enum E1 {
        A,
        B(String),
        C(u32, String) = 5,
        D { f1: String, f2: u32 },
    }

    #[test]
    fn u8_mixed() {
        let t = E1::from(6);
        assert!(matches!(t, E1::D { .. }));
        assert!(matches!(E1::from(5), E1::C(..)));
        assert!(matches!(E1::from(0), E1::A));
        assert!(matches!(E1::from(1), E1::B(_)));
        assert!(matches!(6.into(), E1::D { .. }));
        assert!(matches!(5u8.into(), E1::C(..)));
        assert!(matches!(0u8.into(), E1::A));
        assert!(matches!(1u8.into(), E1::B(_)));
    }

    #[test]
    #[should_panic]
    fn u8_mixed_invalid() {
        let _t: E1 = 7u8.into();
    }
}

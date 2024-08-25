use enum_mapping_macro::U8Mapped;

#[derive(U8Mapped)]
enum Failure {
    A(u8, u8, u8),
    B,
    C { test: String },
}

fn main() {
    let f = Failure::from(4);
}

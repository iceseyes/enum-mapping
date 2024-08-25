# Enum Mapping

Sometimes, it's useful to convert between an enum variant and a numeric code, and vice versa. While this is
straightforward for simple variants (those without fields), it can become tedious when dealing with more complex
variants.

This macro automates the implementation of the `From` and `TryFrom` traits for your enum. If every number in the
specified range is covered by a variant, the macro implements `From`; otherwise, it implements `TryFrom`.

## Usage

```rust
use std::convert::TryInto;
use enum_mapping_macro::U8Mapped;

#[derive(U8Mapped)]
#[repr(u8)]
enum MessageKind {
    Hello,
    Text(String) = 0x10,
    Cfg { version: u8, debug: bool }
}

fn decode_next(buf: &[u8]) -> Result<MessageKind, ()> {
    buf[0].try_into()
}
```

In this example, the macro implements the `TryFrom<u8>` trait for `MessageKind`, mapping `0 => Hello`, `0x10 => Text`,
and `0x11 => Cfg`.

### Mapping Details

- If a variant has a discriminant, that value is used as its mapped value.
- If a variant does not have a discriminant, its value is determined by incrementing the previous variant's value by 1.
  The first variant is always mapped to 0 unless a discriminant is explicitly provided.

Additionally, the macro implements `From<MessageKind>` for `u8`, so you can write code like this:

```rust
use enum_mapping_macro::U8Mapped;

#[derive(U8Mapped)]
#[repr(u8)]
enum MessageKind {
    Hello,
    Text(String) = 0x10,
    Cfg { version: u8, debug: bool }
}

fn test(m: MessageKind) -> u8 {
    m.into()
}
```

### Using a Default Variant

If you prefer to use `From` instead of `TryFrom`, you can define a "catch-all" variant that serves as the default when
no other variant matches:

```rust
use enum_mapping_macro::U8Mapped;
use std::convert::Into;

#[derive(U8Mapped)]
#[repr(u8)]
enum MessageKind {
    Hello,
    Text(String) = 0x10,
    Cfg { version: u8, debug: bool },

    #[catch_all]
    None = 0xff
}

fn decode_next(buf: &[u8]) -> MessageKind {
    buf[0].into()
}
```

In this case, if no specific variant matches the provided value, the `None` variant is returned.


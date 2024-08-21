extern crate core;

mod derive;
mod enum_descriptor;

use crate::derive::derive_from_u8;
use proc_macro::TokenStream;

#[proc_macro_derive(U8Mapped)]
pub fn u8_mapped(item: TokenStream) -> TokenStream {
    derive_from_u8(item.into()).into()
}

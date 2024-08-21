use crate::enum_descriptor::EnumDescriptor;
use proc_macro2::TokenStream;
use quote::quote;

pub fn derive_from_u8(item: TokenStream) -> TokenStream {
    let ast: EnumDescriptor<u8> = syn::parse2(item).unwrap();
    quote!(
        #ast
    )
}

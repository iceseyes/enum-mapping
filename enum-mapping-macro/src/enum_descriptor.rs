use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Expr, Fields, Lit, {Data, DeriveInput},
};

#[derive(Debug)]
pub struct EnumDescriptor {
    name: Ident,
    pairs: Vec<(u8, Ident, usize, Vec<Ident>)>,
}

impl Parse for EnumDescriptor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item_enum: DeriveInput = input.parse()?;
        let ids = match item_enum.data {
            Data::Enum(e) => {
                let mut index = 0u8;
                e.variants
                    .iter()
                    .map(|v| {
                        if let Some((_, Expr::Lit(e))) = &v.discriminant {
                            if let Lit::Int(value) = &e.lit {
                                let r = value.base10_parse::<u8>();
                                if r.is_ok() {
                                    index = r.unwrap()
                                } else {
                                    panic!("value {} out of range for u8", value);
                                }
                            }
                        }

                        let i = index;
                        index += 1;

                        let (unnamed, named) = match &v.fields {
                            Fields::Named(fields) => (
                                0,
                                fields
                                    .named
                                    .iter()
                                    .map(|f| f.clone().ident.unwrap())
                                    .collect(),
                            ),
                            Fields::Unnamed(fields) => (fields.unnamed.len(), vec![]),
                            Fields::Unit => (0, vec![]),
                        };

                        (i, v.ident.clone(), unnamed, named)
                    })
                    .collect()
            }
            _ => panic!("only works for enum(s)"),
        };
        Ok(EnumDescriptor {
            name: item_enum.ident,
            pairs: ids,
        })
    }
}

impl ToTokens for EnumDescriptor {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.name.clone();
        let arms = self.pairs.iter().map(|(i, v, unnamed, named)| {
            let args = if *unnamed > 0 {
                //let args = vec![Expr::new("Default::default()", Span::call_site()); *unnamed];
                //quote! { #(#args,)* }
                let args = std::iter::repeat(quote!(Default::default())).take(*unnamed);
                quote! { (#(#args),*) }
            } else if named.len() > 0 {
                let init = named.iter().map(|f| quote! { #f: Default::default() });
                quote! { { #(#init),* } }
            } else {
                quote! {}
            };
            quote!(#i => Self::#v #args)
        });

        quote!(
            impl From<u8> for #name {
                fn from(value: u8) -> Self {
                    match value {
                        #(#arms,)*
                        _ => panic!("undefined")
                    }
                }
            }
        )
            .to_tokens(tokens)
    }
}

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::any::type_name;
use std::fmt::Display;
use std::ops::AddAssign;
use std::str::FromStr;
use syn::{
    parse::{Parse, ParseStream},
    DataEnum, Expr, Fields, Lit, {Data, DeriveInput},
};

#[derive(Debug)]
pub struct EnumDescriptor<Number> {
    name: Ident,
    pairs: Vec<VariantDescriptor<Number>>,
}

impl<Number> Parse for EnumDescriptor<Number>
where
    Number: AddAssign + FromStr + From<u8> + Copy,
    <Number as FromStr>::Err: Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item_enum: DeriveInput = input.parse()?;
        let ids = match item_enum.data {
            Data::Enum(e) => VariantDescriptor::<Number>::new(e),
            _ => panic!("only works for enum(s)"),
        };

        Ok(EnumDescriptor {
            name: item_enum.ident,
            pairs: ids,
        })
    }
}

impl<Number> ToTokens for EnumDescriptor<Number>
where
    Number: ToTokens + Clone,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let from_enum = FromEnum::<Number>(self);
        let from_u8 = FromNumber::<Number>(self);

        quote!(
            #from_enum
            #from_u8
        )
        .to_tokens(tokens)
    }
}

/// Makes implementation for From<Enum> to u8
struct FromEnum<'a, Number>(&'a EnumDescriptor<Number>);

impl<'a, N> FromEnum<'a, N> {
    fn inner_type_name(&self) -> Ident {
        Ident::new(type_name::<N>(), Span::call_site())
    }
}

impl<'a, Number> ToTokens for FromEnum<'a, Number>
where
    Number: ToTokens + Clone,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.0.name.clone();
        let number = self.inner_type_name();
        let inv_arms = self.0.pairs.iter().map(|vd| {
            let i = vd.id.clone();
            let v = vd.name.clone();
            let args = if vd.unnamed > 0 {
                quote! { ( .. ) }
            } else if vd.fields.len() > 0 {
                quote! { { .. } }
            } else {
                quote! {}
            };
            quote!(#name::#v #args => #i)
        });

        quote!(
            impl From<#name> for #number {
                fn from(value: #name) -> Self {
                    match value {
                        #(#inv_arms,)*
                    }
                }
            }
        )
        .to_tokens(tokens)
    }
}

/// Makes implementation from a Number to Enum
struct FromNumber<'a, Number>(&'a EnumDescriptor<Number>);

impl<'a, N> FromNumber<'a, N> {
    fn inner_type_name(&self) -> Ident {
        Ident::new(type_name::<N>(), Span::call_site())
    }
}

impl<'a, Number> ToTokens for FromNumber<'a, Number>
where
    Number: ToTokens + Clone,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.0.name.clone();
        let number = self.inner_type_name();
        let arms = self.0.pairs.iter().map(|vd| {
            let i = vd.id.clone();
            let v = vd.name.clone();
            let args = if vd.unnamed > 0 {
                let args = std::iter::repeat(quote!(Default::default())).take(vd.unnamed);
                quote! { (#(#args),*) }
            } else if vd.fields.len() > 0 {
                let init = vd.fields.iter().map(|f| quote! { #f: Default::default() });
                quote! { { #(#init),* } }
            } else {
                quote! {}
            };
            quote!(#i => Self::#v #args)
        });

        quote!(
            impl From<#number> for #name {
                fn from(value: #number) -> Self {
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

#[derive(Debug)]
struct VariantDescriptor<Number> {
    id: Number,
    name: Ident,
    unnamed: usize,
    fields: Vec<Ident>,
}

impl<Number> VariantDescriptor<Number>
where
    Number: AddAssign + FromStr + From<u8> + Copy,
    <Number as FromStr>::Err: Display,
{
    fn new(e: DataEnum) -> Vec<Self> {
        let mut index = Number::from(0u8);
        e.variants
            .iter()
            .map(|v| {
                if let Some((_, Expr::Lit(e))) = &v.discriminant {
                    if let Lit::Int(value) = &e.lit {
                        let r = value.base10_parse::<Number>();
                        if r.is_ok() {
                            index = r.unwrap();
                        } else {
                            panic!("value {} out of range for u8", value);
                        }
                    }
                }

                let i = index;
                index += Number::from(1u8);

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

                Self {
                    id: i,
                    name: v.ident.clone(),
                    unnamed,
                    fields: named,
                }
            })
            .collect()
    }
}

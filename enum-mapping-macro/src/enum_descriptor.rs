use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
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
    Number: AddAssign + FromStr + From<u8> + Copy + ToTokens,
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
    Number: ToTokens + Clone + AddAssign + From<u8> + Copy + ToTokens + FromStr,
    <Number as FromStr>::Err: Display,
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
        format_ident!("{}", type_name::<N>())
    }
}

impl<'a, Number> ToTokens for FromEnum<'a, Number>
where
    Number: ToTokens + Clone + AddAssign + From<u8> + Copy + ToTokens + FromStr,
    <Number as FromStr>::Err: Display,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.0.name.clone();
        let number = self.inner_type_name();
        let inv_arms = self
            .0
            .pairs
            .iter()
            .map(|vd| vd.make_match_arm_condition(name));

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
        format_ident!("{}", type_name::<N>())
    }
}

impl<'a, Number> ToTokens for FromNumber<'a, Number>
where
    Number: ToTokens + Clone + AddAssign + From<u8> + Copy + ToTokens + FromStr,
    <Number as FromStr>::Err: Display,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.0.name.clone();
        let number = self.inner_type_name();
        let catch_all = self.0.pairs.iter().find(|vd| vd.catch_all);

        match catch_all {
            Some(catch_all) => {
                let arms = self.0.pairs.iter().map(|vd| vd.make_match_arm(None));
                let catch_all_value = catch_all.make_value();
                quote!(
                    impl From<#number> for #name {
                        fn from(value: #number) -> Self {
                            match value {
                                #(#arms,)*
                                _ => #catch_all_value
                            }
                        }
                    }
                )
            }
            None => {
                let arms = self
                    .0
                    .pairs
                    .iter()
                    .map(|vd| vd.make_match_arm(Some(format_ident!("Ok"))));

                quote!(
                    impl TryFrom<#number> for #name {
                        type Error = ();
                        fn try_from(value: #number) -> Result<Self, Self::Error> {
                            match value {
                                #(#arms,)*
                                _ => Err(())
                            }
                        }
                    }
                )
            }
        }
        .to_tokens(tokens)
    }
}

#[derive(Debug)]
struct VariantDescriptor<Number> {
    id: Number,
    name: Ident,
    unnamed: usize,
    fields: Vec<Ident>,
    catch_all: bool,
}

impl<Number> VariantDescriptor<Number>
where
    Number: AddAssign + FromStr + From<u8> + Copy + ToTokens,
    <Number as FromStr>::Err: Display,
{
    fn new(e: DataEnum) -> Vec<Self> {
        let mut index = Number::from(0u8);
        e.variants
            .iter()
            .map(|v| {
                let catch_all = v
                    .attrs
                    .iter()
                    .find(|a| a.path().is_ident("catch_all"))
                    .is_some();

                if let Some((_, Expr::Lit(e))) = &v.discriminant {
                    if let Lit::Int(value) = &e.lit {
                        let r = value.base10_parse::<Number>();
                        if r.is_ok() {
                            index = r.unwrap();
                        } else {
                            unimplemented!("value {} out of range for u8", value);
                        }
                    }
                }

                let i = index;
                if index < 0xff {
                    index += Number::from(1u8);
                }

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
                    catch_all,
                }
            })
            .collect()
    }

    fn make_value(&self) -> TokenStream {
        let v = self.name.clone();
        let args = if self.unnamed > 0 {
            let args = std::iter::repeat(quote!(Default::default())).take(self.unnamed);
            quote! { (#(#args),*) }
        } else if self.fields.len() > 0 {
            let init = self
                .fields
                .iter()
                .map(|f| quote! { #f: Default::default() });
            quote! { { #(#init),* } }
        } else {
            quote! {}
        };

        quote!(Self::#v #args)
    }

    fn make_match_arm(&self, variant: Option<Ident>) -> TokenStream {
        let i = self.id.clone();
        let v = self.make_value();
        if let Some(name) = variant {
            quote!(#i => #name(#v))
        } else {
            quote!(#i => #v)
        }
    }

    fn make_match_arm_condition(&self, name: &Ident) -> TokenStream {
        let i = self.id.clone();
        let v = self.name.clone();
        let args = if self.unnamed > 0 {
            quote! { ( .. ) }
        } else if self.fields.len() > 0 {
            quote! { { .. } }
        } else {
            quote! {}
        };
        quote!(#name::#v #args => #i)
    }
}

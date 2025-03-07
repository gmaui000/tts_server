/*!
# Enum Ordinalize Derive

This library enables enums to not only obtain the ordinal values of their variants but also allows for the construction of enums from an ordinal value. See the [`enum-ordinalize`](https://crates.io/crates/enum-ordinalize) crate.
*/

mod big_int_wrapper;
mod panic;
mod variant_type;

use std::str::FromStr;

use num_bigint::BigInt;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    Data, DeriveInput, Expr, Fields, Ident, Lit, Meta, Token, UnOp, Visibility,
};
use variant_type::VariantType;

use crate::big_int_wrapper::BigIntWrapper;

#[proc_macro_derive(Ordinalize, attributes(ordinalize))]
pub fn ordinalize_derive(input: TokenStream) -> TokenStream {
    struct ConstMember {
        vis:      Option<Visibility>,
        ident:    Ident,
        meta:     Vec<Meta>,
        function: bool,
    }

    impl Parse for ConstMember {
        #[inline]
        fn parse(input: ParseStream) -> Result<Self, syn::Error> {
            let vis = input.parse::<Visibility>().ok();

            let _ = input.parse::<Token![const]>();

            let function = input.parse::<Token![fn]>().is_ok();

            let ident = input.parse::<Ident>()?;

            let mut meta = Vec::new();

            if !input.is_empty() {
                input.parse::<Token![,]>()?;

                if !input.is_empty() {
                    let result = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

                    let mut has_inline = false;

                    for m in result {
                        if m.path().is_ident("inline") {
                            has_inline = true;
                        }

                        meta.push(m);
                    }

                    if !has_inline {
                        meta.push(syn::parse_str("inline")?);
                    }
                }
            }

            Ok(Self {
                vis,
                ident,
                meta,
                function,
            })
        }
    }

    struct ConstFunctionMember {
        vis:   Option<Visibility>,
        ident: Ident,
        meta:  Vec<Meta>,
    }

    impl Parse for ConstFunctionMember {
        #[inline]
        fn parse(input: ParseStream) -> Result<Self, syn::Error> {
            let vis = input.parse::<Visibility>().ok();

            let _ = input.parse::<Token![const]>();

            input.parse::<Token![fn]>()?;

            let ident = input.parse::<Ident>()?;

            let mut meta = Vec::new();

            if !input.is_empty() {
                input.parse::<Token![,]>()?;

                if !input.is_empty() {
                    let result = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

                    let mut has_inline = false;

                    for m in result {
                        if m.path().is_ident("inline") {
                            has_inline = true;
                        }

                        meta.push(m);
                    }

                    if !has_inline {
                        meta.push(syn::parse_str("inline")?);
                    }
                }
            }

            Ok(Self {
                vis,
                ident,
                meta,
            })
        }
    }

    struct MyDeriveInput {
        ast:                        DeriveInput,
        variant_type:               VariantType,
        values:                     Vec<BigIntWrapper>,
        variant_idents:             Vec<Ident>,
        use_constant_counter:       bool,
        enable_trait:               bool,
        enable_variant_count:       Option<ConstMember>,
        enable_variants:            Option<ConstMember>,
        enable_values:              Option<ConstMember>,
        enable_from_ordinal_unsafe: Option<ConstFunctionMember>,
        enable_from_ordinal:        Option<ConstFunctionMember>,
        enable_ordinal:             Option<ConstFunctionMember>,
    }

    impl Parse for MyDeriveInput {
        fn parse(input: ParseStream) -> Result<Self, syn::Error> {
            let ast = input.parse::<DeriveInput>()?;

            let mut variant_type = VariantType::default();
            let mut enable_trait = cfg!(feature = "traits");
            let mut enable_variant_count = None;
            let mut enable_variants = None;
            let mut enable_values = None;
            let mut enable_from_ordinal_unsafe = None;
            let mut enable_from_ordinal = None;
            let mut enable_ordinal = None;

            for attr in ast.attrs.iter() {
                let path = attr.path();

                if let Some(ident) = path.get_ident() {
                    if ident == "repr" {
                        // #[repr(u8)], #[repr(u16)], ..., etc.
                        if let Meta::List(list) = &attr.meta {
                            let result = list.parse_args_with(
                                Punctuated::<Ident, Token![,]>::parse_terminated,
                            )?;

                            if let Some(value) = result.into_iter().next() {
                                variant_type = VariantType::from_str(value.to_string());
                            }
                        }

                        break;
                    } else if ident == "ordinalize" {
                        if let Meta::List(list) = &attr.meta {
                            let result = list
                                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

                            for meta in result {
                                let path = meta.path();

                                if let Some(ident) = path.get_ident() {
                                    if ident == "impl_trait" {
                                        if let Meta::NameValue(meta) = &meta {
                                            if let Expr::Lit(lit) = &meta.value {
                                                if let Lit::Bool(value) = &lit.lit {
                                                    if cfg!(feature = "traits") {
                                                        enable_trait = value.value;
                                                    }
                                                } else {
                                                    return Err(panic::bool_attribute_usage(
                                                        ident,
                                                        ident.span(),
                                                    ));
                                                }
                                            } else {
                                                return Err(panic::bool_attribute_usage(
                                                    ident,
                                                    ident.span(),
                                                ));
                                            }
                                        } else {
                                            return Err(panic::bool_attribute_usage(
                                                ident,
                                                ident.span(),
                                            ));
                                        }
                                    } else if ident == "variant_count" {
                                        if let Meta::List(list) = &meta {
                                            enable_variant_count = Some(list.parse_args()?);
                                        } else {
                                            return Err(panic::list_attribute_usage(
                                                ident,
                                                ident.span(),
                                            ));
                                        }
                                    } else if ident == "variants" {
                                        if let Meta::List(list) = &meta {
                                            enable_variants = Some(list.parse_args()?);
                                        } else {
                                            return Err(panic::list_attribute_usage(
                                                ident,
                                                ident.span(),
                                            ));
                                        }
                                    } else if ident == "values" {
                                        if let Meta::List(list) = &meta {
                                            enable_values = Some(list.parse_args()?);
                                        } else {
                                            return Err(panic::list_attribute_usage(
                                                ident,
                                                ident.span(),
                                            ));
                                        }
                                    } else if ident == "from_ordinal_unsafe" {
                                        if let Meta::List(list) = &meta {
                                            enable_from_ordinal_unsafe = Some(list.parse_args()?);
                                        } else {
                                            return Err(panic::list_attribute_usage(
                                                ident,
                                                ident.span(),
                                            ));
                                        }
                                    } else if ident == "from_ordinal" {
                                        if let Meta::List(list) = &meta {
                                            enable_from_ordinal = Some(list.parse_args()?);
                                        } else {
                                            return Err(panic::list_attribute_usage(
                                                ident,
                                                ident.span(),
                                            ));
                                        }
                                    } else if ident == "ordinal" {
                                        if let Meta::List(list) = &meta {
                                            enable_ordinal = Some(list.parse_args()?);
                                        } else {
                                            return Err(panic::list_attribute_usage(
                                                ident,
                                                ident.span(),
                                            ));
                                        }
                                    } else {
                                        return Err(panic::sub_attributes_for_ordinalize(
                                            ident.span(),
                                        ));
                                    }
                                } else {
                                    return Err(panic::list_attribute_usage(ident, ident.span()));
                                }
                            }
                        } else {
                            return Err(panic::list_attribute_usage(ident, ident.span()));
                        }
                    }
                }
            }

            let name = &ast.ident;

            if let Data::Enum(data) = &ast.data {
                let variant_count = data.variants.len();

                if variant_count == 0 {
                    return Err(panic::no_variant(name.span()));
                }

                let mut values: Vec<BigIntWrapper> = Vec::with_capacity(variant_count);
                let mut variant_idents: Vec<Ident> = Vec::with_capacity(variant_count);

                let mut use_constant_counter = false;

                if let VariantType::NonDetermined = variant_type {
                    let mut min = BigInt::from(u128::MAX);
                    let mut max = BigInt::from(i128::MIN);
                    let mut counter = BigInt::default();

                    for variant in data.variants.iter() {
                        if let Fields::Unit = variant.fields {
                            let value = if let Some((_, exp)) = variant.discriminant.as_ref() {
                                match exp {
                                    Expr::Lit(lit) => {
                                        if let Lit::Int(value) = &lit.lit {
                                            let value = value.base10_digits();

                                            counter = BigInt::from_str(value).unwrap();

                                            counter.clone()
                                        } else {
                                            return Err(panic::unsupported_discriminant(
                                                lit.span(),
                                            ));
                                        }
                                    },
                                    Expr::Unary(unary) => {
                                        if let UnOp::Neg(_) = unary.op {
                                            match unary.expr.as_ref() {
                                            Expr::Lit(lit) => {
                                                if let Lit::Int(value) = &lit.lit {
                                                    let value = value.base10_digits();

                                                    counter = -BigInt::from_str(value).unwrap();

                                                    counter.clone()
                                                } else {
                                                    return Err(panic::unsupported_discriminant(lit.span()));
                                                }
                                            },
                                            Expr::Path(_)
                                            | Expr::Cast(_)
                                            | Expr::Binary(_)
                                            | Expr::Call(_) => {
                                                return Err(panic::constant_variable_on_non_determined_size_enum(unary.expr.span()))
                                            },
                                            _ => return Err(panic::unsupported_discriminant(unary.expr.span())),
                                        }
                                        } else {
                                            return Err(panic::unsupported_discriminant(
                                                unary.op.span(),
                                            ));
                                        }
                                    },
                                    Expr::Path(_)
                                    | Expr::Cast(_)
                                    | Expr::Binary(_)
                                    | Expr::Call(_) => {
                                        return Err(
                                            panic::constant_variable_on_non_determined_size_enum(
                                                exp.span(),
                                            ),
                                        )
                                    },
                                    _ => return Err(panic::unsupported_discriminant(exp.span())),
                                }
                            } else {
                                counter.clone()
                            };

                            if min > value {
                                min = value.clone();
                            }

                            if max < value {
                                max = value.clone();
                            }

                            variant_idents.push(variant.ident.clone());

                            counter += 1;

                            values.push(BigIntWrapper::from(value));
                        } else {
                            return Err(panic::not_unit_variant(variant.span()));
                        }
                    }

                    if min >= BigInt::from(i8::MIN) && max <= BigInt::from(i8::MAX) {
                        variant_type = VariantType::I8;
                    } else if min >= BigInt::from(i16::MIN) && max <= BigInt::from(i16::MAX) {
                        variant_type = VariantType::I16;
                    } else if min >= BigInt::from(i32::MIN) && max <= BigInt::from(i32::MAX) {
                        variant_type = VariantType::I32;
                    } else if min >= BigInt::from(i64::MIN) && max <= BigInt::from(i64::MAX) {
                        variant_type = VariantType::I64;
                    } else if min >= BigInt::from(i128::MIN) && max <= BigInt::from(i128::MAX) {
                        variant_type = VariantType::I128;
                    } else {
                        return Err(panic::unsupported_discriminant(name.span()));
                    }
                } else {
                    let mut counter = BigInt::default();
                    let mut constant_counter = 0;
                    let mut last_exp: Option<&Expr> = None;

                    for variant in data.variants.iter() {
                        if let Fields::Unit = variant.fields {
                            if let Some((_, exp)) = variant.discriminant.as_ref() {
                                match exp {
                                    Expr::Lit(lit) => {
                                        if let Lit::Int(value) = &lit.lit {
                                            let value = value.base10_digits();

                                            counter = BigInt::from_str(value).unwrap();

                                            values.push(BigIntWrapper::from(counter.clone()));

                                            counter += 1;

                                            last_exp = None;
                                        } else {
                                            return Err(panic::unsupported_discriminant(
                                                lit.span(),
                                            ));
                                        }
                                    },
                                    Expr::Unary(unary) => {
                                        if let UnOp::Neg(_) = unary.op {
                                            match unary.expr.as_ref() {
                                                Expr::Lit(lit) => {
                                                    let lit = &lit.lit;

                                                    match lit {
                                                        Lit::Int(value) => {
                                                            let value = value.base10_digits();

                                                            counter =
                                                                -BigInt::from_str(value).unwrap();

                                                            values.push(BigIntWrapper::from(
                                                                counter.clone(),
                                                            ));

                                                            counter += 1;

                                                            last_exp = None;
                                                        },
                                                        _ => {
                                                            return Err(
                                                                panic::unsupported_discriminant(
                                                                    lit.span(),
                                                                ),
                                                            );
                                                        },
                                                    }
                                                },
                                                Expr::Path(_) => {
                                                    values.push(BigIntWrapper::from((exp, 0)));

                                                    last_exp = Some(exp);
                                                    constant_counter = 1;
                                                },
                                                Expr::Cast(_) | Expr::Binary(_) | Expr::Call(_) => {
                                                    values.push(BigIntWrapper::from((exp, 0)));

                                                    last_exp = Some(exp);
                                                    constant_counter = 1;

                                                    use_constant_counter = true;
                                                },
                                                _ => {
                                                    return Err(panic::unsupported_discriminant(
                                                        exp.span(),
                                                    ));
                                                },
                                            }
                                        } else {
                                            return Err(panic::unsupported_discriminant(
                                                unary.op.span(),
                                            ));
                                        }
                                    },
                                    Expr::Path(_) => {
                                        values.push(BigIntWrapper::from((exp, 0)));

                                        last_exp = Some(exp);
                                        constant_counter = 1;
                                    },
                                    Expr::Cast(_) | Expr::Binary(_) | Expr::Call(_) => {
                                        values.push(BigIntWrapper::from((exp, 0)));

                                        last_exp = Some(exp);
                                        constant_counter = 1;

                                        use_constant_counter = true;
                                    },
                                    _ => return Err(panic::unsupported_discriminant(exp.span())),
                                }
                            } else if let Some(exp) = last_exp {
                                values.push(BigIntWrapper::from((exp, constant_counter)));

                                constant_counter += 1;

                                use_constant_counter = true;
                            } else {
                                values.push(BigIntWrapper::from(counter.clone()));

                                counter += 1;
                            }

                            variant_idents.push(variant.ident.clone());
                        } else {
                            return Err(panic::not_unit_variant(variant.span()));
                        }
                    }
                }

                Ok(MyDeriveInput {
                    ast,
                    variant_type,
                    values,
                    variant_idents,
                    use_constant_counter,
                    enable_trait,
                    enable_variant_count,
                    enable_variants,
                    enable_values,
                    enable_from_ordinal_unsafe,
                    enable_from_ordinal,
                    enable_ordinal,
                })
            } else {
                Err(panic::not_enum(ast.ident.span()))
            }
        }
    }

    // Parse the token stream
    let derive_input = parse_macro_input!(input as MyDeriveInput);

    let MyDeriveInput {
        ast,
        variant_type,
        values,
        variant_idents,
        use_constant_counter,
        enable_trait,
        enable_variant_count,
        enable_variants,
        enable_values,
        enable_ordinal,
        enable_from_ordinal_unsafe,
        enable_from_ordinal,
    } = derive_input;

    // Get the identifier of the type.
    let name = &ast.ident;

    let variant_count = values.len();

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // Build the code
    let mut expanded = proc_macro2::TokenStream::new();

    if enable_trait {
        #[cfg(feature = "traits")]
        {
            let from_ordinal_unsafe = if variant_count == 1 {
                let variant_ident = &variant_idents[0];

                quote! {
                    #[inline]
                    unsafe fn from_ordinal_unsafe(_number: #variant_type) -> Self {
                        Self::#variant_ident
                    }
                }
            } else {
                quote! {
                    #[inline]
                    unsafe fn from_ordinal_unsafe(number: #variant_type) -> Self {
                        ::core::mem::transmute(number)
                    }
                }
            };

            let from_ordinal = if use_constant_counter {
                quote! {
                    #[inline]
                    fn from_ordinal(number: #variant_type) -> Option<Self> {
                        if false {
                            unreachable!()
                        } #( else if number == #values {
                            Some(Self::#variant_idents)
                        } )* else {
                            None
                        }
                    }
                }
            } else {
                quote! {
                    #[inline]
                    fn from_ordinal(number: #variant_type) -> Option<Self> {
                        match number{
                            #(
                                #values => Some(Self::#variant_idents),
                            )*
                            _ => None
                        }
                    }
                }
            };

            expanded.extend(quote! {
                impl #impl_generics Ordinalize for #name #ty_generics #where_clause {
                    type VariantType = #variant_type;

                    const VARIANT_COUNT: usize = #variant_count;

                    const VARIANTS: &'static [Self] = &[#( Self::#variant_idents, )*];

                    const VALUES: &'static [#variant_type] = &[#( #values, )*];

                    #[inline]
                    fn ordinal(&self) -> #variant_type {
                        match self {
                            #(
                                Self::#variant_idents => #values,
                            )*
                        }
                    }

                    #from_ordinal_unsafe

                    #from_ordinal
                }
            });
        }
    }

    let mut expanded_2 = proc_macro2::TokenStream::new();

    if let Some(ConstMember {
        vis,
        ident,
        meta,
        function,
    }) = enable_variant_count
    {
        expanded_2.extend(if function {
            quote! {
                #(#[#meta])*
                #vis const fn #ident () -> usize {
                    #variant_count
                }
            }
        } else {
            quote! {
                #(#[#meta])*
                const #ident: usize = #variant_count;
            }
        });
    }

    if let Some(ConstMember {
        vis,
        ident,
        meta,
        function,
    }) = enable_variants
    {
        expanded_2.extend(if function {
            quote! {
                #(#[#meta])*
                #vis const fn #ident () -> [Self; #variant_count] {
                    [#( Self::#variant_idents, )*]
                }
            }
        } else {
            quote! {
                #(#[#meta])*
                const #ident: [Self; #variant_count] = [#( Self::#variant_idents, )*];
            }
        });
    }

    if let Some(ConstMember {
        vis,
        ident,
        meta,
        function,
    }) = enable_values
    {
        expanded_2.extend(if function {
            quote! {
                #(#[#meta])*
                #vis const fn #ident () -> [#variant_type; #variant_count] {
                    [#( #values, )*]
                }
            }
        } else {
            quote! {
                #(#[#meta])*
                const #ident: [#variant_type; #variant_count] = [#( #values, )*];
            }
        });
    }

    if let Some(ConstFunctionMember {
        vis,
        ident,
        meta,
    }) = enable_from_ordinal_unsafe
    {
        let from_ordinal_unsafe = if variant_count == 1 {
            let variant_ident = &variant_idents[0];

            quote! {
                #(#[#meta])*
                #vis const unsafe fn #ident (_number: #variant_type) -> Self {
                    Self::#variant_ident
                }
            }
        } else {
            quote! {
                #(#[#meta])*
                #vis const unsafe fn #ident (number: #variant_type) -> Self {
                    ::core::mem::transmute(number)
                }
            }
        };

        expanded_2.extend(from_ordinal_unsafe);
    }

    if let Some(ConstFunctionMember {
        vis,
        ident,
        meta,
    }) = enable_from_ordinal
    {
        let from_ordinal = if use_constant_counter {
            quote! {
                #(#[#meta])*
                #vis const fn #ident (number: #variant_type) -> Option<Self> {
                    if false {
                        unreachable!()
                    } #( else if number == #values {
                        Some(Self::#variant_idents)
                    } )* else {
                        None
                    }
                }
            }
        } else {
            quote! {
                #(#[#meta])*
                #vis const fn #ident (number: #variant_type) -> Option<Self> {
                    match number{
                        #(
                            #values => Some(Self::#variant_idents),
                        )*
                        _ => None
                    }
                }
            }
        };

        expanded_2.extend(from_ordinal);
    }

    if let Some(ConstFunctionMember {
        vis,
        ident,
        meta,
    }) = enable_ordinal
    {
        expanded_2.extend(quote! {
            #(#[#meta])*
            #vis const fn #ident (&self) -> #variant_type {
                match self {
                    #(
                        Self::#variant_idents => #values,
                    )*
                }
            }
        });
    }

    if !expanded_2.is_empty() {
        expanded.extend(quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                #expanded_2
            }
        });
    }

    expanded.into()
}

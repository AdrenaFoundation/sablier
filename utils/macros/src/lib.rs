use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use space::{Arg, Args};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, AngleBracketedGenericArguments, DataEnum,
    DataStruct, DeriveInput, Expr, Field, Fields, GenericParam, Generics, PathArguments, Type,
    TypeArray, TypePath, Variant,
};

mod space;

#[proc_macro_derive(MinSpace, attributes(max_len, raw_space))]
pub fn derive_min_space(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let token = match input.data {
        syn::Data::Struct(DataStruct { fields, .. }) => {
            let len_expr = get_len_expr_from_fields(fields);

            quote! {
                #[automatically_derived]
                impl #impl_generics sablier_utils::Space for #name #ty_generics #where_clause {
                    const MIN_SPACE: usize = #len_expr;
                }
            }
        }
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let variants = variants
                .into_iter()
                .map(|Variant { fields, .. }| get_len_expr_from_fields(fields));
            let max = gen_max(variants);

            quote! {
                #[automatically_derived]
                impl sablier_utils::Space for #name {
                    const MIN_SPACE: usize = 1 + #max;
                }
            }
        }
        syn::Data::Union(_) => {
            quote_spanned! { name.span() => compile_error!("Union non implemented.") }
        }
    };

    TokenStream::from(token)
}

// Add a bound `T: Space` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(sablier_utils::Space));
        }
    }
    generics
}

fn gen_max<T: Iterator<Item = TokenStream2>>(mut iter: T) -> TokenStream2 {
    if let Some(item) = iter.next() {
        let next_item = gen_max(iter);
        quote!(sablier_utils::space::max(#item, #next_item))
    } else {
        quote!(0)
    }
}

fn get_len_expr_from_fields(fields: Fields) -> TokenStream2 {
    let len = fields.into_iter().map(|f| match TyLen::try_from(f) {
        Ok(TyLen(len)) => quote!(#len),
        Err(err) => err.into_compile_error(),
    });

    quote!(0 #(+ #len)*)
}

fn expr_from_ty(value: Type, args: &mut Vec<Arg>) -> syn::Result<Expr> {
    let current_arg = args.pop();

    let arg = match current_arg {
        Some(Arg { is_raw, ref value }) => {
            if is_raw {
                return Ok(parse_quote!(#value));
            } else {
                Some(value)
            }
        }
        None => None,
    };

    match value {
        Type::Array(TypeArray { elem, len, .. }) => {
            let inner_ty = expr_from_ty(*elem, args)?;

            Ok(parse_quote!((#len * #inner_ty)))
        }
        Type::Path(TypePath { ref path, .. }) => {
            let Some(segment) = path.segments.last() else {
                return Err(syn::Error::new(value.span(), "Invalid path type."));
            };
            let ident = &segment.ident;

            match ident.to_string().as_str() {
                "String" => {
                    let Some(arg_value) = arg else {
                        return Err(syn::Error::new(ident.span(), "No max_len specified."));
                    };

                    Ok(parse_quote!((4 + #arg_value)))
                }
                "Vec" => {
                    let Some(arg_value) = arg else {
                        return Err(syn::Error::new(ident.span(), "No max_len specified."));
                    };

                    let new_ty = parse_first_arg(&segment.arguments)?;
                    let new_len = expr_from_ty(new_ty, args)?;

                    Ok(parse_quote!((4 + #new_len * #arg_value)))
                }
                _ => Ok(parse_quote!(<#value as sablier_utils::Space>::MIN_SPACE)),
            }
        }
        _ => Ok(parse_quote!(<#value as sablier_utils::Space>::MIN_SPACE)),
    }
}

fn parse_first_arg(path_args: &PathArguments) -> syn::Result<Type> {
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = path_args
    else {
        return Err(syn::Error::new(
            path_args.span(),
            "Invalid type of arguments.",
        ));
    };

    match &args[0] {
        syn::GenericArgument::Type(ty) => Ok(ty.to_owned()),
        _ => Err(syn::Error::new(
            path_args.span(),
            "The first argument is not a type.",
        )),
    }
}

struct TyLen(Expr);

impl TryFrom<Field> for TyLen {
    type Error = syn::Error;

    fn try_from(value: Field) -> Result<Self, Self::Error> {
        let Some(name) = value.ident else {
            return Err(syn::Error::new(value.span(), "Tuple field is not allowed."));
        };

        let mut attr_args = value
            .attrs
            .into_iter()
            .filter_map(|a| Args::try_from(a).ok());

        let args = attr_args.by_ref().take(1).next();

        if attr_args.next().is_some() {
            return Err(syn::Error::new(
                name.span(),
                "max_len and raw_space cannot be used at the same time.",
            ));
        }

        let expr = expr_from_ty(value.ty, &mut args.unwrap_or_default())?;

        Ok(TyLen(expr))
    }
}

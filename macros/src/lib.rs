use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{Data, DataStruct, Fields, Ident, Type};

#[proc_macro_attribute]
pub fn command(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemFn);
    let attr = syn::parse_macro_input!(attr as syn::LitStr);
    let vis = &input.vis;
    let ident = &input.sig.ident;
    let generics = input.sig.generics;

    let mut inputs = input.sig.inputs;
    let arg_count = inputs.iter().len();
    if arg_count < 2 {
        panic!("Command Handler needs at least ctx and sender");
    } else if arg_count == 2 {
        let tokens: TokenStream = "_: crate::commands::from_args::NoArg".parse().unwrap();
        let fn_arg = syn::parse_macro_input!(tokens as syn::FnArg);
        inputs.push(fn_arg);
    }

    let args_arg = match inputs.get(2).expect("Wrong number of arguments") {
        syn::FnArg::Typed(arg) => arg,
        _ => panic!("Wrong type of arguments"),
    };
    let args_type = &args_arg.ty;
    let output = input.sig.output;

    let body = input.block;

    TokenStream::from(quote! {
        #[allow(non_camel_case_types)]
        #vis struct #ident;

        #[async_trait::async_trait]
        impl crate::commands::Command<#args_type> for #ident {
            const NAME: &'static str = #attr;

            async fn handle #generics (#inputs) #output {
                #body
            }
        }
    })
}

fn struct_derive_from_args(ident: &Ident, struct_data: &DataStruct) -> TokenStream2 {
    let (unfold, idents, syntax, type_signature, typed_syntax) = match struct_data.fields {
        Fields::Named(ref fields) => {
            let idents = fields.named.iter().map(|field| &field.ident);
            let types = fields.named.iter().map(|field| &field.ty);
            let type_signature = types
                .clone()
                .map(|ty| ty.to_token_stream().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let syntax = idents
                .clone()
                .map(|ident| match ident {
                    None => unreachable!(),
                    Some(ident) => ident.to_token_stream().to_string(),
                })
                .collect::<Vec<_>>()
                .join(" ");
            let idents = idents
                .map(|ident| ident.to_token_stream())
                .collect::<Vec<_>>();
            let typed_syntax = idents
                .iter()
                .zip(types)
                .map(|(a, b)| format!("    {a}: {}", b.to_token_stream()))
                .collect::<Vec<_>>()
                .join("\n");
            let typed_syntax = format!("{ident} {{\n{typed_syntax}\n}}");
            (
                quote! {Self{ #(#idents),* }},
                idents,
                syntax,
                type_signature,
                typed_syntax,
            )
        }
        Fields::Unnamed(ref fields) => {
            let types = fields
                .unnamed
                .iter()
                .map(|field| &field.ty)
                .collect::<Vec<&Type>>();
            let type_tokens = types
                .iter()
                .map(|ty| ty.to_token_stream().to_string())
                .collect::<Vec<_>>();
            let type_signature = type_tokens.join(", ");
            let syntax = type_tokens.join(" ");

            let arg_count = types.len();
            let placeholders = (0..arg_count)
                .map(|id| format_ident!("p{id}").to_token_stream())
                .collect::<Vec<_>>();

            (
                quote! {Self(#(#placeholders),*)},
                placeholders,
                syntax,
                type_signature.clone(),
                type_signature,
            )
        }
        _ => unimplemented!(),
    };
    let arg_count = idents.len();
    quote! {
        fn from_args(args: Option<&'_ str>) -> crate::common::error::ServiceResult<Self> {
            match args {
                None => Err(crate::common::error::AppError::CommandsInvalidSyntax(Self::SYNTAX, Self::TYPE_SIGNATURE, Self::TYPED_SYNTAX)),
                Some(args) => {
                    let mut parts = args.splitn(#arg_count, ' ');
                    #(
                        let #idents = parts.next();
                    )*
                    #(
                        let #idents = crate::commands::FromCommandArgs::from_args(#idents)
                        .map_err(|e| match e {
                            crate::common::error::AppError::CommandsInvalidSyntax(_,_,_) =>
                            crate::common::error::AppError::CommandsInvalidSyntax(Self::SYNTAX, Self::TYPE_SIGNATURE, Self::TYPED_SYNTAX),
                            e => e,
                        })?;
                    )*
                    Ok(#unfold)
                }
            }
        }

        // TODO: use std::any::type_name::<$t>() when its stabilized
        const TYPE_SIGNATURE: &'static str = concat!(stringify!(#ident), " { ", #type_signature, " }");
        const SYNTAX: &'static str = #syntax;
        const TYPED_SYNTAX: &'static str = #typed_syntax;
    }
}

#[proc_macro_derive(FromCommandArgs)]
pub fn derive_from_args(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let ident = &input.ident;

    let derive_impl = match input.data {
        Data::Struct(ref struct_data) => struct_derive_from_args(ident, struct_data),
        Data::Enum(_) => unimplemented!(),
        Data::Union(_) => unimplemented!(),
    };
    TokenStream::from(quote! {
        impl crate::commands::FromCommandArgs<'_> for #ident {
            #derive_impl
        }
    })
}

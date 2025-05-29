use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn command(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemFn);
    let attr = syn::parse_macro_input!(attr as syn::LitStr);
    let vis = &input.vis;
    let ident = &input.sig.ident;
    let generics = input.sig.generics;
    let inputs = input.sig.inputs;
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

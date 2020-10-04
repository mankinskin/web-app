extern crate syn;
#[macro_use] extern crate quote;
extern crate proc_macro2;
mod rest;
mod rpc;
mod client;
mod server;

use proc_macro::{
    TokenStream,
};
use quote::{
    ToTokens,
};
use proc_macro2::{
    Span,
};
use syn::{
    *,
    Type,
    Macro,
};
struct Items {
    items: Vec<Item>,
}
impl std::ops::Deref for Items {
    type Target = Vec<Item>;
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}
impl syn::parse::Parse for Items {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let mut items = Vec::new();
        while let Ok(item) = input.parse::<Item>() {
            items.push(item);
        }
        Ok(Items { items })
    }
}
struct ItemFns {
    items: Vec<ItemFn>,
}
impl syn::parse::Parse for ItemFns {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let mut items = Vec::new();
        while let Ok(item) = input.parse::<ItemFn>() {
            items.push(item);
        }
        Ok(ItemFns { items })
    }
}

#[proc_macro]
pub fn rest_api(input: TokenStream) -> TokenStream {
    rest::define_rest_api(input)
}
#[proc_macro]
pub fn rest_handlers(input: TokenStream) -> TokenStream {
    let ty = parse_macro_input!(input as Type);
    let ident = Ident::new(
            &format!("{}", ty.clone().into_token_stream()).to_lowercase(),
            Span::call_site(),
        );
    let get_name = format_ident!("get_{}", ident);
    let post_name = format_ident!("post_{}", ident);
    let get_all_name = format_ident!("get_{}s", ident);
    let delete_name = format_ident!("delete_{}", ident);
    let update_name = format_ident!("update_{}", ident);
    TokenStream::from(quote! {
        routes![
            api::handlers::#get_name,
            api::handlers::#post_name,
            api::handlers::#get_all_name,
            api::handlers::#delete_name,
            api::handlers::#update_name,
        ]
    })
}
#[proc_macro]
pub fn api(input: TokenStream) -> TokenStream {
    let items = parse_macro_input!(input as Items);
    let mut fns: Vec<ItemFn> = items
        .iter()
        .filter_map(|item|
            if let Item::Fn(f) = item {
                Some(f.clone())
            } else {
                None
            }
        )
        .collect();
    let imports: Vec<ItemUse> = items
        .iter()
        .filter_map(|item|
            if let Item::Use(u) = item {
                Some(u.clone())
            } else {
                None
            }
        )
        .collect();
    let macros: Vec<ItemMacro> = items
        .iter()
        .filter_map(|item|
            if let Item::Macro(m) = item {
                Some(m.clone())
            } else {
                None
            }
        )
        .collect();
    let rest_apis: Vec<TokenStream> = macros
        .iter()
        .filter_map(|m| {
            let Macro {
                path,
                tokens,
                ..
            } = &m.mac;
            if path.is_ident(&format_ident!("rest_api")) {
                Some(rest::define_rest_api(TokenStream::from(tokens.clone())))
            } else {
                None
            }
        })
        .collect();
    let rest_fns: Vec<Vec<ItemFn>> = rest_apis
        .iter()
        .map(|ts| {
            let fns = syn::parse::<ItemFns>(ts.clone()).unwrap();
            fns.items
        })
        .collect();
    let rest_fns: Vec<ItemFn> = rest_fns[..].concat();
    fns.extend(rest_fns.iter().cloned());

    let protocol = rpc::define_protocol(&fns);
    let server = server::define_server(&fns);
    let client = client::define_client(&fns);
    TokenStream::from(quote! {
        #(#imports)*
        #protocol
        #server
        #client
    })
}

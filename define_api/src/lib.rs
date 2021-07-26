mod client;
mod rest;
mod rpc;
mod server;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{
    format_ident,
    quote,
    ToTokens,
};
use syn::{
    Macro,
    Type,
    *,
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

/// Define server side REST handlers for a type
#[proc_macro]
pub fn rest_api(input: TokenStream) -> TokenStream {
    rest::define_rest_api(input)
}
/// Define server REST endpoints for a type
#[proc_macro]
pub fn rest_handlers(input: TokenStream) -> TokenStream {
    // input is a Type
    let ty = parse_macro_input!(input as Type);
    // to lowercase Ident
    let ident = Ident::new(
        &format!("{}", ty.clone().into_token_stream()).to_lowercase(),
        Span::call_site(),
        );
    // rest method idents
    let get_name = format_ident!("get_{}", ident);
    let post_name = format_ident!("post_{}", ident);
    let get_all_name = format_ident!("get_{}s", ident);
    let delete_name = format_ident!("delete_{}", ident);
    //let update_name = format_ident!("update_{}", ident);
    TokenStream::from(quote! {
        rocket::routes![
            api::handlers::#get_name,
            api::handlers::#post_name,
            api::handlers::#get_all_name,
            api::handlers::#delete_name,
            //api::handlers::#update_name,
        ]
    })
}
#[proc_macro]
pub fn api(input: TokenStream) -> TokenStream {
    // parse Items
    let items = parse_macro_input!(input as Items);
    // filter functions
    let mut fns: Vec<ItemFn> = items
        .iter()
        .filter_map(|item| {
            if let Item::Fn(f) = item {
                Some(f.clone())
            } else {
                None
            }
        })
    .collect();
    // filter imports
    let imports: Vec<ItemUse> = items
        .iter()
        .filter_map(|item| {
            if let Item::Use(u) = item {
                Some(u.clone())
            } else {
                None
            }
        })
    .collect();
    // filter macros
    let macros: Vec<ItemMacro> = items
        .iter()
        .filter_map(|item| {
            if let Item::Macro(m) = item {
                Some(m.clone())
            } else {
                None
            }
        })
    .collect();
    // execute rest_api macros
    let rest_apis: Vec<TokenStream> = macros
        .iter()
        .filter_map(|m| {
            let Macro { path, tokens, .. } = &m.mac;
            if path.is_ident(&format_ident!("rest_api")) {
                Some(rest::define_rest_api(TokenStream::from(tokens.clone())))
            } else {
                None
            }
        })
    .collect();
    // parse rest function items
    let rest_fns: Vec<ItemFn> = rest_apis
        .iter()
        .flat_map(|ts| {
            let fns = syn::parse::<ItemFns>(ts.clone()).unwrap();
            fns.items
        })
    .collect();
    // append rest functions to other functions
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

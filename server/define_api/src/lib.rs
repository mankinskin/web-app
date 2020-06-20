extern crate syn;
#[macro_use] extern crate quote;
extern crate proc_macro2;

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
    parse_macro_input,
    Ident,
    Type,
    export::{
        TokenStream2,
    },
};
#[proc_macro]
pub fn define_api(input: TokenStream) -> TokenStream {
    let ty = parse_macro_input!(input as Type);
    let ident = Ident::new(
            &format!("{}", ty.clone().into_token_stream()).to_lowercase(),
            Span::call_site(),
        );
    let get = define_get(ty.clone(), ident.clone());
    let get_all = define_get_all(ty.clone(), ident.clone());
    let post = define_post(ty.clone(), ident.clone());
    let delete = define_delete(ty.clone(), ident.clone());
    TokenStream::from(quote! {
        #get
        #get_all
        #post
        #delete
    })
}
#[proc_macro]
pub fn define_api_routes(input: TokenStream) -> TokenStream {
    let ty = parse_macro_input!(input as Type);
    let ident = Ident::new(
            &format!("{}", ty.clone().into_token_stream()).to_lowercase(),
            Span::call_site(),
        );
    let ident_plural = format_ident!("{}s", ident);
    let get_name = format_ident!("get_{}", ident);
    let post_name = format_ident!("post_{}", ident);
    let get_all_name = format_ident!("get_{}", ident_plural);
    let delete_name = format_ident!("delete_{}", ident);
    TokenStream::from(quote! {
        routes![
            #get_name,
            #post_name,
            #get_all_name,
            #delete_name,
        ]
    })
}

fn define_get(ty: Type, ident: Ident) -> TokenStream2 {
    let ident_plural = format_ident!("{}s", ident);
    let route = format!("/api/{}/<id>", ident_plural);
    let name = format_ident!("get_{}", ident);
    quote! {
        #[get(#route)]
        fn #name(id: SerdeParam<Id<#ty>>) -> Json<Option<#ty>> {
            Json(#ty::get(*id))
        }
    }
}
fn define_post(ty: Type, ident: Ident) -> TokenStream2 {
    let ident_plural = format_ident!("{}s", ident);
    let route = format!("/api/{}", ident_plural);
    let name = format_ident!("post_{}", ident);
    quote! {
        #[post(#route, data="<data>")]
        fn #name(data: Json<#ty>) -> Json<Id<#ty>> {
            Json(#ty::insert(data.clone()))
        }
    }
}
fn define_get_all(ty: Type, ident: Ident) -> TokenStream2 {
    let ident_plural = format_ident!("{}s", ident);
    let route = format!("/api/{}", ident_plural);
    let name = format_ident!("get_{}", ident_plural);
    quote! {
        #[get(#route)]
        fn #name() -> Json<Vec<Entry<#ty>>> {
            Json(#ty::get_all())
        }
    }
}
fn define_delete(ty: Type, ident: Ident) -> TokenStream2 {
    let ident_plural = format_ident!("{}s", ident);
    let route = format!("/api/{}/<id>", ident_plural);
    let name = format_ident!("delete_{}", ident);
    quote! {
        #[delete(#route)]
        fn #name(id: SerdeParam<Id<#ty>>) -> Json<Option<#ty>> {
            Json(#ty::delete(id.clone()))
        }
    }
}

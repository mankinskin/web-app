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
    *,
    Type,
    Macro,
    punctuated::{
        Punctuated,
    },
    token::{
        *
    },
    export::{
        TokenStream2,
    },
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
                Some(define_rest_api(TokenStream::from(tokens.clone())))
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
    let protocols: Vec<TokenStream2> = fns
        .iter()
        .map(|f| rpc_protocol(f.clone()))
        .collect();
    let clients: Vec<TokenStream2> = fns
        .iter()
        .map(|f| rpc_client(f.clone()))
        .collect();
    let calls: Vec<TokenStream2> = fns
        .iter()
        .map(|f| rpc_call(f.clone()))
        .collect();
    let routes: Vec<TokenStream2> = fns
        .iter()
        .map(|f| rpc_route(f.clone()))
        .collect();
    TokenStream::from(quote! {
        use serde::{
            Serialize,
            Deserialize,
        };
        use seed::{
            browser::fetch::{
                Method,
                FetchError,
            },
        };
        use rocket::{
            request::{
                FromParam,
            },
        };
        use std::result::{
            Result,
        };
        use rocket_contrib::{
            json::{
                Json,
            },
        };
        use updatable::{
            Updatable
        };
        #(#imports)*

        #(#protocols)*
        #(#clients)*
        #[cfg(not(target_arch="wasm32"))]
        mod call {
            use super::*;
            #(#calls)*
        }
        #[cfg(not(target_arch="wasm32"))]
        pub mod routes {
            use super::*;
            #(#routes)*
        }
    })
}
fn rpc_route(item: ItemFn) -> TokenStream2 {
    #[allow(unused)]
    let ItemFn {
        attrs,      //: Vec<Attribute>
        vis,        //: Visibility
        sig,       //: Signature
        block,      //: Box<Block>
    } = item.clone();

    #[allow(unused)]
    let Signature {
        constness,  //: Option<Const>
        asyncness,  //: Option<Async>
        unsafety,   //: Option<Unsafe>
        abi,        //: Option<Abi>
        fn_token,   //: Fn
        ident,      //: Ident
        generics,   //: Generics
        paren_token,//: Paren
        inputs,     //: Punctuated<FnArg, Comma>
        variadic,   //: Option<Dot3>
        output,     //: ReturnType
    } = sig;

    let params_ident = format_ident!("{}Parameters", ident.clone());
    let result_ident = format_ident!("{}Result", ident.clone());

    let route = format!("/api/call/{}", ident);
    let args: Punctuated<Expr, Comma> = inputs.iter().map(|arg| {
            match arg {
                FnArg::Typed(ty) => {
                    let member = format!("parameters.{}",
                            match *ty.pat.clone() {
                                Pat::Ident(pat) => pat.ident,
                                _ => panic!("api function params must have idents"),
                            }
                        );
                    let expr: ExprField = syn::parse_str(&member).unwrap();
                    Expr::Field(expr)
                },
                _ => panic!("api functions may not take self parameter"),
            }
        }).collect();
    quote! {
        #[post(#route, data="<parameters>")]
        pub fn #ident(parameters: Json<#params_ident>) -> Json<#result_ident> {
            let Json(parameters) = parameters;
            Json(#result_ident(call::#ident(#args)))
        }
    }
}
fn rpc_protocol(item: ItemFn) -> TokenStream2 {
    let Signature {
        ident,      //: Ident
        inputs,     //: Punctuated<FnArg, Comma>
        output,     //: ReturnType
        ..
    } = item.sig;

    let params_ident = format_ident!("{}Parameters", ident.clone());
    let params = rpc_params(params_ident.clone(), inputs.clone());

    let result_ident = format_ident!("{}Result", ident.clone());
    let result = rpc_result(result_ident.clone(), output.clone());

    quote! {
        #params
        #result
    }
}
fn rpc_client(item: ItemFn) -> TokenStream2 {
    let Signature {
        ident,      //: Ident
        inputs,     //: Punctuated<FnArg, Comma>
        output,     //: ReturnType
        ..
    } = item.sig;

    let params_ident = format_ident!("{}Parameters", ident.clone());

    let result_ident = format_ident!("{}Result", ident.clone());

    let route = format!("/api/call/{}", ident);
    let members: Punctuated<Ident, Comma> = inputs.iter().map(|arg| {
            match arg {
                FnArg::Typed(ty) => {
                    match *ty.pat.clone() {
                        Pat::Ident(pat) => pat.ident,
                        _ => panic!("api function params must have idents"),
                    }
                },
                _ => panic!("api functions may not take self parameter"),
            }
        }).collect();

    let ret_ty: Type = match output {
        ReturnType::Default => { syn::parse_str("()").unwrap() },
        ReturnType::Type(_arrow, ty) => {
            *ty
        },
    };
    quote! {
        #[cfg(target_arch="wasm32")]
        pub async fn #ident(#inputs) -> Result<#ret_ty, FetchError> {
            let url = format!("{}{}", "http://localhost:8000", #route);
            // authentication
            //if let Some(session) = root::get_session() {
            //    req = req.header(Header::authorization(format!("{}", session.token)));
            //}
            seed::fetch::fetch(
                seed::fetch::Request::new(&url)
                    .method(Method::Post)
                    .json(&#params_ident { #members })?
            )
            .await?
            .check_status()?
            .json()
            .await
            .map(|res: #result_ident| res.0)
        }
    }
}
fn rpc_call(item: ItemFn) -> TokenStream2 {
    let mut item = item;
    item.vis = Visibility::Public(
        VisPublic {
            pub_token: Pub::default(),
        }
    );
    quote! {
        #item
    }
}
fn rpc_params(ident: Ident, inputs: Punctuated<FnArg, Comma>) -> TokenStream2 {
    let fields = Fields::Named(FieldsNamed {
        brace_token: Brace::default(),
        named: inputs.iter().map(|arg| {
            match arg {
                FnArg::Typed(ty) =>
                    Field {
                        attrs: ty.attrs.clone(),
                        vis: Visibility::Inherited,
                        ident: Some(
                            match *ty.pat.clone() {
                                Pat::Ident(pat) => pat.ident,
                                _ => panic!("api function params must have idents"),
                            }
                        ),
                        colon_token: Some(ty.colon_token),
                        ty: *ty.ty.clone(),
                    },
                _ => panic!("api functions may not take self parameter"),
            }
        }).collect()
    });
    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Deserialize, Serialize)]
        pub struct #ident #fields
    }
}
fn rpc_result(ident: Ident, ty: ReturnType) -> TokenStream2 {
    let fields = match ty {
        ReturnType::Default => { Fields::Unit },
        ReturnType::Type(_arrow, ty) => {
            let mut fields = Punctuated::new();
            fields.push_value(Field {
                attrs: vec![],
                vis: Visibility::Inherited,
                ident: None,
                colon_token: None,
                ty: *ty,
            });
            Fields::Unnamed(FieldsUnnamed {
                paren_token: Paren::default(),
                unnamed: fields,
            })
        },
    };
    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Deserialize, Serialize)]
        pub struct #ident #fields;
    }
}
fn define_rest_api(input: TokenStream) -> TokenStream {
    let ty = parse_macro_input!(input as Type);
    let ident = Ident::new(
            &format!("{}", ty.clone().into_token_stream()).to_lowercase(),
            Span::call_site(),
        );
    let get = define_get(ty.clone(), ident.clone());
    let get_all = define_get_all(ty.clone(), ident.clone());
    let post = define_post(ty.clone(), ident.clone());
    let delete = define_delete(ty.clone(), ident.clone());
    let _update = define_update(ty.clone(), ident.clone());
    TokenStream::from(quote! {
        #get
        #get_all
        #post
        #delete
    })
}
#[proc_macro]
pub fn rest_api(input: TokenStream) -> TokenStream {
    define_rest_api(input)
}
#[proc_macro]
pub fn rest_api_routes(input: TokenStream) -> TokenStream {
    let ty = parse_macro_input!(input as Type);
    let ident = Ident::new(
            &format!("{}", ty.clone().into_token_stream()).to_lowercase(),
            Span::call_site(),
        );
    let get_name = format_ident!("get_{}", ident);
    let post_name = format_ident!("post_{}", ident);
    let get_all_name = format_ident!("get_{}s", ident);
    let delete_name = format_ident!("delete_{}", ident);
    let _update_name = format_ident!("update_{}", ident);
    TokenStream::from(quote! {
        routes![
            api::routes::#get_name,
            api::routes::#post_name,
            api::routes::#get_all_name,
            api::routes::#delete_name,
        ]
    })
}

fn define_get(ty: Type, ident: Ident) -> TokenStream2 {
    let name = format_ident!("get_{}", ident);
    quote! {
        fn #name(id: Id<#ty>) -> Option<#ty> {
            #ty::get(id)
        }
    }
}
fn define_post(ty: Type, ident: Ident) -> TokenStream2 {
    let name = format_ident!("post_{}", ident);
    quote! {
        fn #name(data: #ty) -> Id<#ty> {
            #ty::insert(data)
        }
    }
}
fn define_get_all(ty: Type, ident: Ident) -> TokenStream2 {
    let name = format_ident!("get_{}s", ident);
    quote! {
        fn #name() -> Vec<Entry<#ty>> {
            #ty::get_all()
        }
    }
}
fn define_delete(ty: Type, ident: Ident) -> TokenStream2 {
    let name = format_ident!("delete_{}", ident);
    quote! {
        fn #name(id: Id<#ty>) -> Option<#ty> {
            #ty::delete(id)
        }
    }
}
fn define_update(ty: Type, ident: Ident) -> TokenStream2 {
    let name = format_ident!("update_{}", ident);
    quote! {
        fn #name(id: Id<#ty>, update: <#ty as Updatable>::Update) -> Option<#ty> {
            #ty::update(id, update)
        }
    }
}

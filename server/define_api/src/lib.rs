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
struct ItemFns {
    items: Vec<ItemFn>,
}
impl syn::parse::Parse for ItemFns {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let mut items = Vec::new();
        while let Ok(item) = input.parse::<ItemFn>() {
            items.push(item);
        }
        Ok(ItemFns{ items })
    }
}
#[proc_macro]
pub fn api(input: TokenStream) -> TokenStream {
    let fns = parse_macro_input!(input as ItemFns);
    let definitions: Vec<TokenStream2> = fns
        .items
        .iter()
        .map(|item| rpc_definitions(item.clone()))
        .collect();
    let calls: Vec<TokenStream2> = fns
        .items
        .iter()
        .map(|item| rpc_call(item.clone()))
        .collect();
    let routes: Vec<TokenStream2> = fns
        .items
        .iter()
        .map(|item| rpc_route(item.clone()))
        .collect();
    TokenStream::from(quote! {
        #(#definitions)*
        #[cfg(not(target_arch="wasm32"))]
        mod call {
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
fn rpc_definitions(item: ItemFn) -> TokenStream2 {
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
    let params = rpc_params(params_ident.clone(), inputs.clone());

    let result_ident = format_ident!("{}Result", ident.clone());
    let result = rpc_result(result_ident.clone(), output.clone());

    let client = rpc_client(
        ident.clone(),
        params_ident.clone(),
        result_ident.clone(),
        output.clone(),
        inputs.clone(),
    );
    quote! {
        #params
        #result
        #client
    }
}
fn rpc_client(
    ident: Ident,
    params_ident: Ident,
    result_ident: Ident,
    result_ty: ReturnType,
    inputs: Punctuated<FnArg, Comma>,
) -> TokenStream2
{
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

    let ret_ty: Type = match result_ty {
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

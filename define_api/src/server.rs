use quote::{
	quote,
	format_ident,
};
use syn::{
	export::TokenStream2,
	punctuated::Punctuated,
	token::*,
	*,
};

pub fn define_server(fns: &Vec<ItemFn>) -> TokenStream2 {
	let handlers = define_handlers(&fns);
	let calls = define_calls(&fns);
	quote! {
		#handlers
		#calls
	}
}
fn define_calls(fns: &Vec<ItemFn>) -> TokenStream2 {
	let calls: Vec<TokenStream2> = fns.iter().map(|f| call(f.clone())).collect();

	quote! {
		#[cfg(not(target_arch="wasm32"))]
		mod call {
			use super::*;
			#(#calls)*
		}
	}
}
fn call(item: ItemFn) -> TokenStream2 {
	let mut item = item;
	item.vis = Visibility::Public(VisPublic {
		pub_token: Pub::default(),
	});
	quote! {
		#item
	}
}
fn define_handlers(fns: &Vec<ItemFn>) -> TokenStream2 {
	let routes: Vec<TokenStream2> = fns.iter().map(|f| route(f.clone())).collect();
	quote! {
		#[cfg(not(target_arch="wasm32"))]
		pub mod handlers {
			use super::*;
			use jwt::{
				*,
			};
			use rocket::{
				request::{
					FromParam,
				},
				response::{
					*,
				},
				http::{
					*,
				},
			};
			use rocket_contrib::{
				json::{
					Json,
				},
			};
			#(#routes)*
		}
	}
}
fn route(item: ItemFn) -> TokenStream2 {
	#[allow(unused)]
	let ItemFn {
		attrs, //: Vec<Attribute>
		vis,   //: Visibility
		sig,   //: Signature
		block, //: Box<Block>
	} = item.clone();

	#[allow(unused)]
	let Signature {
		constness,   //: Option<Const>
		asyncness,   //: Option<Async>
		unsafety,	//: Option<Unsafe>
		abi,		 //: Option<Abi>
		fn_token,	//: Fn
		ident,	   //: Ident
		generics,	//: Generics
		paren_token, //: Paren
		inputs,	  //: Punctuated<FnArg, Comma>
		variadic,	//: Option<Dot3>
		output,	  //: ReturnType
	} = sig;

	let params_ident = format_ident!("{}Parameters", ident.clone());
	let result_ident = format_ident!("{}Result", ident.clone());

	let route = format!("/api/call/{}", ident);
	let args: Punctuated<Expr, Comma> = inputs
		.iter()
		.map(|arg| {
			match arg {
				FnArg::Typed(ty) => {
					let member = format!(
						"parameters.{}",
						match *ty.pat.clone() {
							Pat::Ident(pat) => pat.ident,
							_ => panic!("api function params must have idents"),
						}
					);
					let expr: ExprField = syn::parse_str(&member).unwrap();
					Expr::Field(expr)
				}
				_ => panic!("api functions may not take self parameter"),
			}
		})
		.collect();
	quote! {
		#[rocket::post(#route, data="<parameters>")]
		pub fn #ident(token: JWT, parameters: Json<#params_ident>) -> Json<#result_ident> {
			let _ = token;
			let Json(parameters) = parameters;
			Json(#result_ident(call::#ident(#args)))
		}
	}
}

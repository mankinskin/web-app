use proc_macro::TokenStream;
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::{
	format_ident,
	quote,
	ToTokens,
};
use syn::{
	Type,
	*,
};
pub fn define_rest_api(input: TokenStream) -> TokenStream {
	let ty = parse_macro_input!(input as Type);
	let ident = Ident::new(
		&format!("{}", ty.clone().into_token_stream()).to_lowercase(),
		Span::call_site(),
	);
	let get = define_get(ty.clone(), ident.clone());
	let get_all = define_get_all(ty.clone(), ident.clone());
	let post = define_post(ty.clone(), ident.clone());
	let delete = define_delete(ty.clone(), ident.clone());
	//let update = define_update(ty.clone(), ident.clone());
	TokenStream::from(quote! {
		#get
		#get_all
		#post
		#delete
		//#update
	})
}
fn define_get(ty: Type, ident: Ident) -> TokenStream2 {
	let name = format_ident!("get_{}", ident);
	quote! {
		fn #name(id: Id<#ty>) -> Option<Entry<#ty>> {
			<#ty as DatabaseTable<'_, Schema>>::get(id)
		}
	}
}
fn define_post(ty: Type, ident: Ident) -> TokenStream2 {
	let name = format_ident!("post_{}", ident);
	quote! {
		fn #name(data: #ty) -> Id<#ty> {
			<#ty as DatabaseTable<'_, Schema>>::insert(data)
		}
	}
}
fn define_get_all(ty: Type, ident: Ident) -> TokenStream2 {
	let name = format_ident!("get_{}s", ident);
	quote! {
		fn #name() -> Vec<Entry<#ty>> {
			<#ty as DatabaseTable<'_, Schema>>::get_all()
		}
	}
}
fn define_delete(ty: Type, ident: Ident) -> TokenStream2 {
	let name = format_ident!("delete_{}", ident);
	quote! {
		fn #name(id: Id<#ty>) -> Option<#ty> {
			<#ty as DatabaseTable<'_, Schema>>::delete(id)
		}
	}
}
//fn define_update(ty: Type, ident: Ident) -> TokenStream2 {
//	let name = format_ident!("update_{}", ident);
//	quote! {
//		fn #name(id: Id<#ty>, update: <#ty as Updatable>::Update) -> Option<#ty> {
//			<#ty as DatabaseTable>::update(id, update)
//		}
//	}
//}

#![feature(specialization)]
extern crate syn;
#[macro_use] extern crate quote;
extern crate proc_macro2;

use proc_macro::{
    TokenStream,
};
use proc_macro2::{
    TokenTree,
    Literal,
};
use syn::{
    parse_macro_input,
    DeriveInput,
    Ident,
    Field,
    Fields,
    FieldsNamed,
    Data,
    Type,
    export::{
        TokenStream2,
    },
};

#[proc_macro_derive(Updatable)]
pub fn derive_updatable(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let updatable_impl = updatable_impl(input.clone());
    let update_struct = update_struct(input.clone());
    let update_impl = update_impl(input);
    TokenStream::from(quote! {
        #updatable_impl
        #update_struct
        #update_impl
    })
}
fn update_struct(input: DeriveInput) -> syn::export::TokenStream2 {
    let DeriveInput {
        data,
        vis,
        ident,
        attrs: _,
        generics: _,
    } = input.clone();
    let update_ident = format_ident!("{}Update", ident.clone());
    let update_builder = update_builder(input.clone());
    let update_data = update_data(data);
    quote! {
        #[derive(Clone, Default, Debug, PartialEq)]
        #vis struct #update_ident #update_data
        #update_builder
    }
}
fn update_data(data: Data) -> syn::export::TokenStream2 {
    match data {
        Data::Struct(data) => {
            let fields = update_fields(data.fields);
            let semi = data.semi_token;
            quote! { #fields #semi }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
fn update_builder(input: DeriveInput) -> syn::export::TokenStream2 {
    let ident = input.ident.clone();
    let update_ident = format_ident!("{}Update", ident.clone());
    let update_setters =
        match input.data {
            Data::Struct(data) => update_setters(data.fields),
            Data::Enum(_) | Data::Union(_) => unimplemented!(),
        };
    quote! {
        impl #ident {
            pub fn update() -> #update_ident {
               <#update_ident as Default>::default()
            }
        }
        impl #update_ident {
            pub fn builder() -> #update_ident {
                Self::default()
            }
            #update_setters
        }
    }
}
fn updatable_impl(input: DeriveInput) -> TokenStream2 {
    let ident = input.ident.clone();
    let update_ident = format_ident!("{}Update", ident.clone());
    quote! {
        impl Updatable for #ident {
            type Update = #update_ident;
        }
    }
}
fn update_impl(input: DeriveInput) -> TokenStream2 {
    let ident = input.ident.clone();
    let update_ident = format_ident!("{}Update", ident.clone());
    let update_calls =
        match input.data {
            Data::Struct(data_struct) => update_calls(data_struct.fields),
            Data::Enum(_) | Data::Union(_) => unimplemented!(),
        };
    quote! {
        impl Update<#ident> for #update_ident {
            fn update(&self, data: &mut #ident) {
                #update_calls
            }
        }
    }
}
fn update_field(field: &mut Field) {
    let ty = field.ty.clone();
    field.ty = syn::parse2(quote!{
        Option<<#ty as Updatable>::Update>
    }).unwrap();
}
fn update_fields(fields: Fields) -> Fields {
    match fields.clone() {
        Fields::Named(mut inner) => {
            inner.named
                .iter_mut()
                .for_each(update_field);
            Fields::Named(inner)
        },
        Fields::Unnamed(mut inner) => {
            inner.unnamed
                .iter_mut()
                .for_each(update_field);
            Fields::Unnamed(inner)
        },
        Fields::Unit => {
            Fields::Unit
        },
    }
}
fn update_setters(fields: Fields) -> TokenStream2 {
    let field_ident = field_idents(fields.clone());
    let setter_ident = setter_idents(fields.clone());
    let field_ty = field_types(fields.clone());
    quote! {
        #(pub fn #setter_ident(self, update: <#field_ty as Updatable>::Update) -> Self {
                let mut new = self;
                new.#field_ident = Some(update);
                new
            })*
    }
}
fn update_calls(fields: Fields) -> TokenStream2 {
    let field_ident = field_idents(fields);
    quote! {
        #(self.#field_ident.clone().map(|update| update.update(&mut data.#field_ident));)*
    }
}
fn setter_idents(fields: Fields) -> Vec<TokenTree> {
    match fields {
        Fields::Named(_) => {
            field_idents(fields)
        },
        Fields::Unnamed(inner) => {
            inner.unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    TokenTree::Ident(format_ident!("field_{}", i))
                })
                .collect()
        },
        Fields::Unit => {
            Vec::new()
        }
    }
}
fn named_field_idents(fields: FieldsNamed) -> Vec<Ident> {
    fields.named
        .iter()
        .map(|field| {
            field.ident.clone().unwrap()
        })
        .collect()
}
fn field_idents(fields: Fields) -> Vec<TokenTree> {
    match fields {
        Fields::Named(inner) => {
            named_field_idents(inner)
                .iter()
                .map(|ident| TokenTree::Ident(format_ident!("{}", ident.to_string())))
                .collect()
        },
        Fields::Unnamed(inner) => {
            inner.unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    TokenTree::Literal(Literal::usize_unsuffixed(i))
                })
                .collect()
        },
        Fields::Unit => {
            Vec::new()
        }
    }
}
fn field_types(fields: Fields) -> Vec<Type> {
    match fields.clone() {
        Fields::Named(inner) => {
            inner.named
                .iter()
                .map(|field| {
                    field.ty.clone()
                })
                .collect()
        },
        Fields::Unnamed(inner) => {
            inner.unnamed
                .iter()
                .map(|field| {
                    field.ty.clone()
                })
                .collect()
        },
        Fields::Unit => {
            Vec::new()
        },
    }
}


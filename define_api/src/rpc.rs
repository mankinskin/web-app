use syn::{
    export::TokenStream2,
    punctuated::Punctuated,
    token::*,
    *,
};

pub fn define_protocol(fns: &Vec<ItemFn>) -> TokenStream2 {
    let protocols: Vec<TokenStream2> = fns.iter().map(|f| protocol(f.clone())).collect();
    quote! {
        use serde::{
            Serialize,
            Deserialize,
        };
        use std::result::{
            Result,
        };
        use updatable::{
            Updatable
        };
        use app_model::{
            user::*,
            auth::*,
        };
        #(#protocols)*
    }
}
fn protocol(item: ItemFn) -> TokenStream2 {
    let Signature {
        ident,  //: Ident
        inputs, //: Punctuated<FnArg, Comma>
        output, //: ReturnType
        ..
    } = item.sig;

    let params_ident = format_ident!("{}Parameters", ident.clone());
    let params = params(params_ident.clone(), inputs.clone());

    let result_ident = format_ident!("{}Result", ident.clone());
    let result = result(result_ident.clone(), output.clone());

    quote! {
        #params
        #result
    }
}
fn params(ident: Ident, inputs: Punctuated<FnArg, Comma>) -> TokenStream2 {
    let fields = Fields::Named(FieldsNamed {
        brace_token: Brace::default(),
        named: inputs
            .iter()
            .map(|arg| {
                match arg {
                    FnArg::Typed(ty) => {
                        Field {
                            attrs: ty.attrs.clone(),
                            vis: Visibility::Inherited,
                            ident: Some(match *ty.pat.clone() {
                                Pat::Ident(pat) => pat.ident,
                                _ => panic!("api function params must have idents"),
                            }),
                            colon_token: Some(ty.colon_token),
                            ty: *ty.ty.clone(),
                        }
                    }
                    _ => panic!("api functions may not take self parameter"),
                }
            })
            .collect(),
    });
    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Deserialize, Serialize)]
        pub struct #ident #fields
    }
}
fn result(ident: Ident, ty: ReturnType) -> TokenStream2 {
    let fields = match ty {
        ReturnType::Default => Fields::Unit,
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
        }
    };
    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Deserialize, Serialize)]
        pub struct #ident #fields;
    }
}

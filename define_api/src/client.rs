use quote::{
    format_ident,
    quote,
};
use syn::{
    export::TokenStream2,
    punctuated::Punctuated,
    token::*,
    Type,
    *,
};

pub fn define_client(fns: &Vec<ItemFn>) -> TokenStream2 {
    let calls: Vec<TokenStream2> = fns.iter().map(|f| fetch_call(f.clone())).collect();
    quote! {
        #[cfg(target_arch="wasm32")]
        use seed::{
            browser::fetch::{
                Method,
                FetchError,
                Header,
                Response,
            },
        };
        use app_model::{
            auth,
        };
        #(#calls)*
    }
}
fn fetch_call(item: ItemFn) -> TokenStream2 {
    let Signature {
        ident,  //: Ident
        inputs, //: Punctuated<FnArg, Comma>
        output, //: ReturnType
        ..
    } = item.sig;

    let params_ident = format_ident!("{}Parameters", ident.clone());
    let result_ident = format_ident!("{}Result", ident.clone());
    let route = format!("/api/call/{}", ident);
    let members: Punctuated<Ident, Comma> = inputs
        .iter()
        .map(|arg| {
            match arg {
                FnArg::Typed(ty) => {
                    match *ty.pat.clone() {
                        Pat::Ident(pat) => pat.ident,
                        _ => panic!("api function params must have idents"),
                    }
                }
                _ => panic!("api functions may not take self parameter"),
            }
        })
        .collect();
    let ret_ty: Type = match output {
        ReturnType::Default => syn::parse_str("()").unwrap(),
        ReturnType::Type(_arrow, ty) => *ty,
    };
    quote! {
        #[cfg(target_arch="wasm32")]
        pub async fn #ident(#inputs) -> Result<#ret_ty, FetchError> {
            let host = "http://localhost:8000";
            let url = format!("{}{}", host, #route);
            let mut req = seed::fetch::Request::new(&url)
                .method(Method::Post);
            // authentication
            if let Some(session) = auth::session::get() {
                req = req.header(Header::authorization(format!("{}", session.token)));
            }
            seed::fetch::fetch(
                req.json(&#params_ident { #members })?
            )
            .await?
            .check_status()?
            .json()
            .await
            .map(|res: #result_ident| res.0)
        }
    }
}

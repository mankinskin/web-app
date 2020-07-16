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

pub fn define_client(fns: &Vec<ItemFn>) -> TokenStream2 {
    let calls: Vec<TokenStream2> = fns
        .iter()
        .map(|f| fetch_call(f.clone()))
        .collect();
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
        // TODO, implement through api
        #[cfg(target_arch="wasm32")]
        pub async fn register(user: User) -> Result<UserSession, FetchError> {
            let url = format!("{}{}", "http://localhost:8000", "/api/auth/register");
            let mut req = seed::fetch::Request::new(&url)
                .method(Method::Post);
            seed::fetch::fetch(
                req.json(&user)?
            )
            .await?
            .check_status()?
            .json()
            .await
            .map(|session: UserSession| session)
        }
        #[cfg(target_arch="wasm32")]
        pub async fn login(credentials: Credentials) -> Result<UserSession, FetchError> {
            let url = format!("{}{}", "http://localhost:8000", "/api/auth/login");
            let mut req = seed::fetch::Request::new(&url)
                .method(Method::Post);
            seed::fetch::fetch(
                req.json(&credentials)?
            )
            .await?
            .check_status()?
            .json()
            .await
            .map(|session: UserSession| session)
        }
        #(#calls)*
    }
}
fn fetch_call(item: ItemFn) -> TokenStream2 {
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
            let host = "http://localhost:8000";
            let url = format!("{}{}", host, #route);
            let mut req = seed::fetch::Request::new(&url)
                .method(Method::Post);
            // authentication
            if let Some(session) = auth::get_session() {
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

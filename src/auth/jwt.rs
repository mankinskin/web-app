use crate::user::*;
use chrono::{Duration, Utc};
use jsonwebtoken::{errors::*, DecodingKey, EncodingKey, Header, *};
use lazy_static::lazy_static;
use serde::*;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct JWTProvider {
    secret: String,
    header: Header,
    validation: Validation,
}
lazy_static! {
    static ref JWT_PROVIDER: JWTProvider = JWTProvider::new();
    static ref TOKEN_LIFETIME: Duration = Duration::days(1);
}

fn get_token_lifetime() -> Duration {
    *TOKEN_LIFETIME
}
impl JWTProvider {
    fn generate_secret() -> String {
        //format!("{}", chrono::Utc::now().timestamp_nanos())
        format!("{}", "dw8ed832ekjdkal29ewi92ie921e09iksdjwjwi")
    }
    pub fn new() -> Self {
        let validation = Validation {
            validate_exp: true,
            validate_nbf: true,
            leeway: 60,
            ..Default::default()
        };
        Self {
            secret: Self::generate_secret(),
            header: Header::default(),
            validation,
        }
    }
    pub fn encode(&self, claims: &JWTClaims) -> Result<String> {
        encode(
            &self.header,
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }
    pub fn decode(&self, token: &str) -> Result<TokenData<JWTClaims>> {
        decode(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &self.validation,
        )
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JWTClaims {
    sub: String,
    exp: i64,
    iat: i64,
    nbf: i64,
}
impl From<&User> for JWTClaims {
    fn from(user: &User) -> Self {
        Self {
            sub: user.name().clone(),
            iat: Utc::now().timestamp(),
            nbf: Utc::now().timestamp(),
            exp: (Utc::now() + get_token_lifetime()).timestamp(),
        }
    }
}
#[derive(Debug)]
pub enum JWTError {
    MissingToken,
    Invalid(jsonwebtoken::errors::Error),
    BadCount,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct JWT(String);

impl Display for JWT {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl TryFrom<&User> for JWT {
    type Error = errors::Error;
    fn try_from(user: &User) -> std::result::Result<Self, Self::Error> {
        let claims = JWTClaims::from(user);
        JWT::encode(&claims)
    }
}
impl JWT {
    pub fn encode(claims: &JWTClaims) -> Result<Self> {
        JWT_PROVIDER.encode(&claims).map(JWT::from)
    }
    pub fn decode(&self) -> Result<JWTClaims> {
        JWT_PROVIDER.decode(&self.0).map(|td| td.claims)
    }
}
impl From<String> for JWT {
    fn from(s: String) -> Self {
        Self(s)
    }
}
use rocket::{
    request::{
        FromRequest,
        Request,
        Outcome,
    },
    http::Status,
};
impl<'a, 'r> FromRequest<'a, 'r> for JWT {
    type Error = JWTError;
    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("authorization").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, JWTError::MissingToken)),
            1 => {
                let token = keys[0];
                match JWT_PROVIDER.decode(token) {
                    Ok(_claims) => Outcome::Success(JWT(token.to_string())),
                    Err(err) => Outcome::Failure((Status::BadRequest, JWTError::Invalid(err))),
                }
            },
            _ => Outcome::Failure((Status::BadRequest, JWTError::BadCount)),
        }
    }
}
#[cfg(test)]
pub mod tests {
    use super::*;
    lazy_static! {
        static ref JWT_PROVIDER: JWTProvider = JWTProvider::new();
        static ref TOKEN_LIFETIME: Duration = Duration::minutes(5);
    }
    #[test]
    fn encode_decode() {
        let user = User::new("Slim Shady", "my_name_is");
        let claims = JWTClaims::from(&user);
        let token = JWT::encode(&claims).unwrap();
        assert_eq!(token.decode().unwrap(), claims)
    }
}

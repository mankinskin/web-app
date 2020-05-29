use crate::{
    user::*,
};
use jsonwebtoken::{
    *,
    Header,
    errors::*,
};
use rocket::{
    http::{
        *,
    },
    request::{
        *,
    },
};

#[derive(
    Debug,
    )]
pub struct JWTProvider {
    secret: String,
    header: Header,
    validation: Validation,
}
lazy_static! {
    static ref JWT_PROVIDER: JWTProvider = JWTProvider::new();
}

impl JWTProvider {
    fn generate_secret() -> String {
        format!("{}", chrono::Utc::now().timestamp_nanos())
    }
    pub fn new() -> Self {
        let validation = Validation {
            validate_exp: true,
            validate_iat: true,
            validate_nbf: true,
            ..Default::default()
        };
        Self {
            secret: Self::generate_secret(),
            header: Header::default(),
            validation,
        }
    }
    pub fn encode(&self, claims: &JWTClaims) -> Result<String> {
        encode(&self.header, &claims, &self.secret.as_bytes())
    }
    pub fn decode(&self, token: &str) -> Result<TokenData<JWTClaims>> {
        decode(token, &self.secret.as_bytes(), &self.validation)
    }
    pub fn token_is_valid(&self, token: &str) -> bool {
        self.decode(token).is_ok()
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    )]
pub struct JWTClaims {
    sub: String,
}
impl From<&User> for JWTClaims {
    fn from(user: &User) -> Self {
        Self {
            sub: user.name().clone(),
        }
    }
}
#[derive(
    Debug,
    )]
pub enum JWTError {
    Encoding(Error),
    MissingToken,
    Invalid,
    BadCount,
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Serialize,
    Deserialize,
    )]
pub struct JWT(String);

impl JWT {
    pub fn new_for(user: &User) -> Result<Self> {
        let claims = JWTClaims::from(user);
        JWT_PROVIDER.encode(&claims).map(JWT::from)
    }
}
impl From<String> for JWT {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl<'a, 'r> FromRequest<'a, 'r> for JWT {
    type Error = JWTError;
    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, JWTError::MissingToken)),
            1 if JWT_PROVIDER.token_is_valid(keys[0])
                => Outcome::Success(JWT(keys[0].to_string())),
            1 => Outcome::Failure((Status::BadRequest, JWTError::Invalid)),
            _ => Outcome::Failure((Status::BadRequest, JWTError::BadCount)),
        }
    }
}

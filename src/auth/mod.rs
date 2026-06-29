pub mod callback_server;
pub mod tiktok_auth;
pub mod token_store;

pub use token_store::TokenData;

#[derive(Debug)]
pub enum AuthResult {
    Token(TokenData),
    Error(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum AuthStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

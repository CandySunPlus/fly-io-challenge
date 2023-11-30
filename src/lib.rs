pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub mod node;
pub mod protocol;

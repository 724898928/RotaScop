pub mod protocol;
pub use protocol::*;
pub type Result<T> = std::result::Result<T,String>;
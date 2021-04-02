use alloc::string::String;
use scale::{Decode, Encode};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Encode, Decode, Debug, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    InvalidArgument(String),
    IllegalMove(String),
    InvalidCaller,
    Other,
}

impl core::convert::From<ink_env::Error> for Error {
    fn from(_: ink_env::Error) -> Self {
        Self::Other
    }
}

impl core::convert::From<core::fmt::Error> for Error {
    fn from(_: core::fmt::Error) -> Self {
        Self::Other
    }
}

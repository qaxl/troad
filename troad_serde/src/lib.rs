mod de;
mod err;
mod ser;

/// Used for #[serde(with = "var_int")]
pub mod var_int;

pub use de::from_slice;
pub use ser::{to_vec, to_vec_with_size};
pub use err::{Result, Error};

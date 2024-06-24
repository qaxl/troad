mod de;
mod err;
mod ser;
mod var_int;

pub use de::{deserialize_from_slice, Deserializer, SizedVec};
pub use ser::{serialize_to_vec, serialize_with_size, Serializer};
pub use var_int::{v32, v64, vsize, VarInt};

// TODO: fix, this is a temp "hack"
use var_int::VarIntVisitor;

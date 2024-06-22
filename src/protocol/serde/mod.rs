mod de;
mod err;
mod ser;
mod var_int;

pub use de::{Deserializer, deserialize_from_slice};
pub use ser::{Serializer, serialize};
pub use var_int::{VarI32, VarI64, VarInt};

use std::marker::PhantomData;

use serde::{
    de::{self, Visitor},
    ser, Deserializer, Serializer,
};

pub mod macros {
    macro_rules! var_int_ser_impl {
        ($serializer:ident, $input:expr) => {
            {
                use cast::From as _0;
                use serde::ser::SerializeSeq;

                let mut value: u64 =
                u64::cast($input).map_err(|_| ser::Error::custom("value isn't convertible to i64"))?;
                // More efficient would be to use `serialize_bytes`, but our custom serializer would result up in a cyclic dependency.
                let mut seq = $serializer.serialize_seq(None)?;

                loop {
                    if (value & !0x7F) == 0 {
                        seq.serialize_element(&(value as u8))?;
                        break;
                    }

                    seq.serialize_element(&((value & 0x7F | 0x80) as u8))?;
                    value >>= 7;
                }

                seq.end()
            }
        };
    }

    pub(crate) use var_int_ser_impl;
}

// New recommended way
pub fn serialize<S, T>(input: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Copy,
    u64: cast::From<T, Output = Result<u64, cast::Error>>,
{
    macros::var_int_ser_impl!(serializer, *input)
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: cast::From<u64, Output = Result<T, cast::Error>>,
{
    deserializer.deserialize_seq(VarIntVisitor::<T>(PhantomData))
}

pub(crate) struct VarIntVisitor<T>(pub(super) PhantomData<T>);

impl<'de, T: cast::From<u64, Output = Result<T, cast::Error>>> Visitor<'de> for VarIntVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a var int")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut value: u64 = 0;
        let mut position: u64 = 0;

        loop {
            let byte = seq
                .next_element::<u8>()?
                .ok_or_else(|| de::Error::invalid_length((position / 7) as usize, &self))?;

            let byte = byte as u64;

            value |= (byte & 0x7F) << position;

            if (byte & 0x80) == 0 {
                return T::cast(value).map_err(|_| de::Error::custom("var int is too big"));
            }

            position += 7;

            if position >= (std::mem::size_of::<T>() * 7) as u64 {
                return Err(de::Error::custom("var int is too long! (>64 bytes)"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[test]
    fn var_int_de_ser() {
        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct StructWithVarInts {
            #[serde(with = "crate::var_int")]
            x: i32,
            #[serde(with = "crate::var_int")]
            y: isize,
            #[serde(with = "crate::var_int")]
            z: u128, // just a test, you NEVER should use u128 in prod, because it's not really supported.
        }

        let stct = StructWithVarInts {
            x: 42949,
            y: isize::MAX,
            z: 455434355,
        };

        // VarInts... in JSON. YEEHAWWWWWW!
        let json = serde_json::to_string_pretty(&stct).unwrap();

        // println!("{json}");

        assert_eq!(
            serde_json::from_str::<StructWithVarInts>(&json).unwrap(),
            stct
        );
    }
}

use std::{marker::PhantomData, ops::Deref};

use serde::{
    de::{self, Visitor},
    ser, Deserialize, Deserializer, Serialize, Serializer,
};

#[macro_export]
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

// New recommended way
pub fn serialize<S, T>(input: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Copy,
    u64: cast::From<T, Output = Result<u64, cast::Error>>,
{
    var_int_ser_impl!(serializer, *input)
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: cast::From<u64, Output = Result<T, cast::Error>>,
{
    deserializer.deserialize_seq(VarIntVisitor::<T>(PhantomData))
}

pub(super) struct VarIntVisitor<T>(pub(super) PhantomData<T>);

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

            if position >= (std::mem::size_of::<T>() * 8) as u64 {
                return Err(de::Error::custom("var int is too long! (>64 bytes)"));
            }
        }
    }
}

macro_rules! var_int_impl {
    ($type:ty, $name:ident, $visitor:ident) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(pub $type);

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut bytes = [0u8; 10];
                let mut length = 0;

                let mut value = self.0;

                for b in &mut bytes {
                    length += 1;

                    if (value & !0x7F) == 0 {
                        *b = (value & 0xF) as u8;
                        break;
                    }

                    *b = (value & 0x7F | 0x80) as u8;
                    value >>= 7;
                }

                serializer.serialize_bytes(&bytes[..length])
            }
        }

        #[derive(Default)]
        struct $visitor(PhantomData<$type>);

        impl<'de> Visitor<'de> for $visitor {
            type Value = $name;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a var int")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut value: $type = 0;
                let mut position: $type = 0;

                loop {
                    let byte = seq
                        .next_element::<u8>()?
                        .ok_or_else(|| de::Error::invalid_length((position / 7) as usize, &self))?;

                    let byte = byte as $type;

                    value |= (byte & 0x7F) << position;

                    if (byte & 0x80) == 0 {
                        return Ok($name(value));
                    }

                    position += 7;

                    if position >= (std::mem::size_of::<$type>() * 8) as $type {
                        return Err(de::Error::custom("var int is too long! (>64 bytes)"));
                    }
                }
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_seq($visitor(PhantomData))
            }
        }

        impl From<$type> for $name {
            fn from(value: $type) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $type {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl Deref for $name {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// Old way of serialization...
var_int_impl!(i32, VarInt, VarIntVisitor2);
var_int_impl!(i64, VarLong, VarLongVisitor);
var_int_impl!(usize, VarSize, VarSizeVisitor);

// Only the new way is tested
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

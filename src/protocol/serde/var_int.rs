use std::{fmt::Display, marker::PhantomData, ops::Deref};

use num::FromPrimitive;
use num_traits::PrimInt;
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};

// Quick and dirty implementation of variable-length integers for any size.
// Minecraft, only officially supports i32 and i64, but we can use this for our own purposes too.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VarInt<T: PrimInt>(pub T);

// Hereby, let's declare the "official" ones here.
pub type v32 = VarInt<i32>;
pub type v64 = VarInt<i64>;

// all other var int types are supposed to be signed, but because rust uses `usize`
// in so many other places for buffer sizes, this is an exception and is using `usize` instead of `isize`.
pub type vsize = VarInt<usize>;

// After this line of code, I have absolutely idea why and how it works.
// Improvements are welcome, this was thrown together by me with no experience in serde.

impl<T: PrimInt> Serialize for VarInt<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut bytes = [0u8; 10];
        let mut length = 0;

        // I mean... Do we need to support i128?
        // Well, if we do. Just change this, I guess.
        let mut value = self
            .0
            .to_i64()
            .expect("a variable-length integer couldn't be represented as a 64-bit integer.");

        for b in &mut bytes {
            length += 1;

            if (value & !0x7F) == 0 {
                *b = value as u8;
                break;
            }

            *b = (value & 0x7F | 0x80) as u8;
            value >>= 7;
        }

        serializer.serialize_bytes(&bytes[..length])
    }
}

struct VarIntVisitor<T>(PhantomData<T>);

impl<'de, T: PrimInt + FromPrimitive> Visitor<'de> for VarIntVisitor<T> {
    type Value = VarInt<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a variable-length integer (LEB128, standard not zigzag)")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut value: i64 = 0;
        let mut position: i64 = 0;

        // println!("decoding var int");

        loop {
            let byte = seq
                .next_element::<u8>()?
                .ok_or_else(|| de::Error::invalid_length((position / 7) as usize, &self))?;

            let byte = byte as i64;

            value |= (byte & 0x7F) << position;

            if (byte & 0x80) == 0 {
                return Ok(VarInt::<T>(FromPrimitive::from_i64(value).ok_or_else(
                    || de::Error::custom("can't convert from i64 to current type... too large?"),
                )?));
            }

            position += 7;

            if position >= (std::mem::size_of::<T>() * 8) as i64 {
                return Err(de::Error::custom("varint is too long! (>64 bytes)"));
            }
        }
    }
}

impl<'de, T: PrimInt + FromPrimitive> Deserialize<'de> for VarInt<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(VarIntVisitor::<T>(PhantomData))
    }
}

impl From<i32> for VarInt<i32> {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<VarInt<i32>> for i32 {
    fn from(value: VarInt<i32>) -> Self {
        value.0
    }
}

impl<T: PrimInt> Deref for VarInt<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: PrimInt + Display> std::fmt::Display for VarInt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

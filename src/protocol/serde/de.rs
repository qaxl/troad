#![allow(dead_code)]
#![allow(unused_variables)]

// TODO: ^^^ implement these for every type

use std::marker::PhantomData;

use serde::{
    de::{self, DeserializeSeed, SeqAccess, Visitor},
    Deserialize,
};

use crate::protocol::serde::var_int::VarIntVisitor;

use super::{
    err::{Error, Result},
    vsize, VarInt,
};

pub struct Deserializer<'de> {
    data: Slice<'de>,
}

impl<'de> Deserializer<'de> {
    pub fn from_slice(input: &'de [u8]) -> Self {
        Self {
            data: Slice::new(input),
        }
    }

    pub fn size(&self) -> usize {
        self.data.size()
    }
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // You can read it though...
    fn is_human_readable(&self) -> bool {
        false
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.data.try_take_n(1)?[0] != 0)
    }

    fn deserialize_any<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("won't implement");
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.data.try_take_n(1)?[0])
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let input = self.data.try_take_n(8)?;
        // This literally shouldn't fail.
        visitor.visit_i64(i64::from_be_bytes(input.try_into().unwrap()))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.data.try_take_n(1)?[0])
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let slice = self.data.try_take_n(2)?;
        visitor.visit_u16(u16::from_be_bytes([slice[0], slice[1]]))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let slice = self.data.try_take_n(8)?;
        visitor.visit_u64(u64::from_be_bytes(slice[0..8].try_into().unwrap()))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let slice = self.data.try_take_n(4)?;
        // minecraft protocol is :sparkles: special and i don't think they do any bitwise shifting on floats.
        visitor.visit_f32(f32::from_be_bytes(slice[0..4].try_into().unwrap()))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let slice = self.data.try_take_n(8)?;
        // minecraft protocol is :sparkles: special and i don't think they do any bitwise shifting on floats.
        visitor.visit_f64(f64::from_be_bytes(slice[0..8].try_into().unwrap()))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = VarInt::<i32>::deserialize(&mut *self)?.0 as usize;
        let slice = self.data.try_take_n(len)?;
        let str = core::str::from_utf8(slice).map_err(|_| Error::BadUtf8Input)?;

        visitor.visit_borrowed_str(str)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.size();

        // The VarInt implementation is kinda complicated, so this is a temporary "fix".
        // Just allow it to read as fucking many bytes as it wants in a sequence.
        visitor.visit_seq(SeqAccessImpl { de: self, len })
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqAccessImpl { de: self, len })
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }
}

struct SeqAccessImpl<'a, 'b: 'a> {
    de: &'a mut Deserializer<'b>,
    len: usize,
}

impl<'a, 'b: 'a> SeqAccess<'b> for SeqAccessImpl<'a, 'b> {
    type Error = Error;

    fn next_element_seed<V: DeserializeSeed<'b>>(&mut self, seed: V) -> Result<Option<V::Value>> {
        if self.len > 0 {
            self.len -= 1;
            Ok(Some(DeserializeSeed::deserialize(seed, &mut *self.de)?))
        } else {
            Ok(None)
        }
    }
}

// This is heavily inspired by the postcard implementation.
// https://github.com/jamesmunns/postcard/blob/main/source/postcard/src/de/flavors.rs
struct Slice<'a> {
    pub(crate) cur: *const u8,
    pub(crate) end: *const u8,
    pub(crate) _pd: PhantomData<&'a [u8]>,
}

unsafe impl<'a> Send for Slice<'a> {}

impl<'a> Slice<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        unsafe {
            Self {
                cur: slice.as_ptr(),
                end: slice.as_ptr().add(slice.len()),
                _pd: PhantomData,
            }
        }
    }

    pub fn size(&self) -> usize {
        unsafe { self.end.offset_from(self.cur) as usize }
    }

    pub fn try_take_n(&mut self, n: usize) -> Result<&'a [u8]> {
        let remaining = self.size();
        // println!("{remaining}");
        if remaining < n {
            Err(Error::Eof)
        } else {
            unsafe {
                let slice = core::slice::from_raw_parts(self.cur, n);
                self.cur = self.cur.add(n);
                Ok(slice)
            }
        }
    }
}

pub fn deserialize_from_slice<T: for<'a> Deserialize<'a>>(slice: &[u8]) -> Result<(usize, T)> {
    let mut deserializer = Deserializer::from_slice(slice);
    let data = T::deserialize(&mut deserializer)?;

    Ok((slice.len() - deserializer.size(), data))
}

#[derive(Debug)]
pub struct SizedVec(pub vsize, pub Vec<u8>);

struct SizedVecVisitor;
impl<'de> Visitor<'de> for SizedVecVisitor {
    type Value = SizedVec;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a SizedVec")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let size = seq.next_element::<vsize>()?.unwrap();
        let mut vec = vec![0; *size];
        for v in &mut vec {
            *v = seq.next_element::<u8>()?.unwrap();
        }

        Ok(SizedVec(size, vec))
    }
}

impl<'de> Deserialize<'de> for SizedVec {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_tuple_struct("", usize::MAX, SizedVecVisitor)
    }
}

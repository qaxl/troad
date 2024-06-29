#![allow(dead_code)]
#![allow(unused_variables)]

// TODO: ^^^ implement these for every type

use std::marker::PhantomData;

use serde::{
    de::{self, DeserializeSeed, EnumAccess, SeqAccess, VariantAccess, Visitor},
    Deserialize,
};

use crate::var_int;

use super::err::{Error, Result};

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
        visitor.visit_i16(i16::from_be_bytes(self.data.try_take_n_exact(2)?))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
       visitor.visit_i32(i32::from_be_bytes(self.data.try_take_n_exact(4)?))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(i64::from_be_bytes(self.data.try_take_n_exact(8)?))
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
        visitor.visit_u16(u16::from_be_bytes(self.data.try_take_n_exact(2)?))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(u32::from_be_bytes(self.data.try_take_n_exact(4)?))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(u64::from_be_bytes(self.data.try_take_n_exact(8)?))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(f32::from_be_bytes(self.data.try_take_n_exact(4)?))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(f64::from_be_bytes(self.data.try_take_n_exact(8)?))
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
        let len = self.deserialize_seq(var_int::VarIntVisitor::<u64>(PhantomData))? as usize;
        let slice = self.data.try_take_n(len)?;
        let str = core::str::from_utf8(slice).map_err(|_| Error::BadUtf8Input)?;

        visitor.visit_borrowed_str(str)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.deserialize_seq(var_int::VarIntVisitor::<u64>(PhantomData))? as usize;
        let slice = self.data.try_take_n(len)?;

        visitor.visit_bytes(slice)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // As far as I am aware, binary doesn't really let you make "copyless" Vec...
        self.deserialize_bytes(visitor)
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
        visitor.visit_enum(EnumAccessImpl { de: self })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let index = self.deserialize_seq(var_int::VarIntVisitor::<u32>(PhantomData))?;
        visitor.visit_u32(index)
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

struct EnumAccessImpl<'a, 'b: 'a> {
    de: &'a mut Deserializer<'b>,
}

impl<'a, 'b: 'a> EnumAccess<'b> for EnumAccessImpl<'a, 'b> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'b>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for EnumAccessImpl<'a, 'de> {
    type Error = Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<()> {
        Ok(())
        // Err(Error::ExpectedString)
    }

    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }
}

// This is heavily inspired by the postcard implementation. Thanks to them, I was able to create a safe interface for slices.
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
        if remaining < n {
            Err(Error::Eof)
        } else {
            // SAFETY: the length of the slice is checked before trying to return it to the callee
            unsafe {
                let slice = core::slice::from_raw_parts(self.cur, n);
                self.cur = self.cur.add(n);
                Ok(slice)
            }
        }
    }

    pub fn try_take_n_exact<const N: usize>(&mut self, n: usize) -> Result<[u8; N]> {
        let remaining = self.size();
        if remaining < n {
            Err(Error::Eof)
        } else {
            // SAFETY: the length of the slice is checked before trying to return it to the callee
            unsafe {
                let slice = self.cur as *const [u8; N];
                self.cur = self.cur.add(n);
                Ok(*slice)
            }
        }
    }
}

/// Deserializes data from slice. 
/// The returning value is (read_size, deserialized_data).
/// # NOTE:
/// This function has this weird return value because the main library user (currently closed source, `troad`) may call `from_slice` multiple times.
/// *This may change in the future and the function might get a normal return value*
pub fn from_slice<T: for<'a> Deserialize<'a>>(slice: &[u8]) -> Result<(usize, T)> {
    let mut deserializer = Deserializer::from_slice(slice);
    let data = T::deserialize(&mut deserializer)?;

    Ok((slice.len() - deserializer.size(), data))
}

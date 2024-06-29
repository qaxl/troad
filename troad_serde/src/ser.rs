use serde::{
    ser,
    Serialize,
};

use super::{err::{Error, Result}, var_int::macros::var_int_ser_impl};

#[derive(Debug)]
pub struct Serializer {
    output: Vec<u8>,
}

impl Serializer {
    // yea i know, 10 bytes extra... don't be so dramatic bro
    pub fn new() -> Self {
        Self {
            output: Vec::with_capacity(16),
        }
    }

    pub fn output(self) -> Vec<u8> {
        self.output
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.output.push(v as u8);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.output.push(v as u8);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.output.extend(v.to_be_bytes().iter());
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.output.extend(v.to_be_bytes().iter());
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.output.extend(v.to_be_bytes().iter());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.output.extend(v.to_be_bytes().iter());
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.output.extend(v.to_be_bytes().iter());
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.output.extend(v.to_be_bytes().iter());
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.output.extend(v.to_be_bytes().iter());
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.output.extend(v.to_be_bytes().iter());
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.output.extend(v.to_be_bytes().iter());
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        let mut encoded = [0; 4];
        let encoded = v.encode_utf8(&mut encoded);
        self.output.extend(encoded.as_bytes());
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        var_int_ser_impl!(self, v.len())?;
        self.output.extend(v.as_bytes());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        var_int_ser_impl!(self, v.len())?;
        self.output.extend(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
    ) -> Result<Self::Ok> {
        var_int_ser_impl!(self, variant_index)?;
        Ok(())
    }

    fn serialize_newtype_struct<T>(self, _: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        var_int_ser_impl!(self, variant_index)?;
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        var_int_ser_impl!(self, variant_index)?;
        self.serialize_tuple(len)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
        unimplemented!("not supported by minecraft protocol")
    }

    fn serialize_struct(self, _: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        var_int_ser_impl!(self, variant_index)?;
        self.serialize_tuple(len)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

/// Serializes `data` into a `Vec<u8>`, 
/// the returned `Vec<u8>` has the length prefixed in front as a variable-length integer.
/// See `to_vec` if you don't want the data to be length-prefixed.
pub fn to_vec_with_size<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    #[derive(Serialize)]
    struct LengthPrefixedData {
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    }

    let mut serializer = Serializer::new();
    data.serialize(&mut serializer)?;

    let data = LengthPrefixedData {
        data: serializer.output,
    };

    let mut serializer = Serializer::new();
    data.serialize(&mut serializer)?;

    Ok(serializer.output)
}

/// Serializes `data` into a `Vec<u8>`.
/// See `to_vec_with_size` if you want to have length-prefixed data.
pub fn to_vec<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    let mut serializer = Serializer::new();
    data.serialize(&mut serializer)?;

    Ok(serializer.output)
}

/// Generic module for binary serialization of messages exchanged in the Tezos p2p protocol.
/// Note: currently minimally implemented to just do the handshake.
/// Note: Very inefficient: serde (or I can't figure out a way) doesn't offer an obivous
/// way for handling fixed sized arrays during serialization.
///
use std::{io, usize};

use serde::{
    de::{self, SeqAccess, Visitor},
    forward_to_deserialize_any,
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Deserialize, Deserializer, Serialize, Serializer,
};

use super::error::{Error, Result};

/// Serializes plain data in the packed binary format of Tezos P2P protocol, the data is fully returned
/// and ready to be sent to peers as is.
/// Returns a buffer of concatenated (header,msg)
/// where:
/// header: 2 bytes, u16 = length of the remaining msg.
/// msg: tezos encoding of the message
/// - port, protocol_version, distributed_db_version: u16
/// - public_key: 32 bytes
/// - nonces: 24 bytes
/// - string: (u32 length, string in bytes)
///
pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut output = vec![0u8; 2];
    output.reserve(1024);

    let mut ser = TezosBinSerializer { output, length: 0 };
    value.serialize(&mut ser)?;
    ser.output[0] = (ser.length >> 8) as u8;
    ser.output[1] = (ser.length & 0xff) as u8;
    Ok(ser.output)
}
pub fn to_bytes_no_header<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut output = vec![0u8; 0];
    output.reserve(1024);

    let mut ser = TezosBinSerializer { output, length: 0 };
    value.serialize(&mut ser)?;
    Ok(ser.output)
}

/// Binary serializer for Tezos
///
/// Rather inefficient but necessary to capture the data before we send it specially
/// during the handshake to compute nonces for NaCl encrypting and signature.
struct TezosBinSerializer {
    output: Vec<u8>,
    length: u16,
}

impl TezosBinSerializer {
    /// increments length and check for overflow
    fn incr_length(&mut self, added: u16) -> Result<()> {
        if self.length + added < self.length {
            Err(Error::SizeOverflow {
                before: self.length,
                added,
            })
        } else {
            self.length += added;
            Ok(())
        }
    }
}

impl<'a> Serializer for &'a mut TezosBinSerializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        if v.len() >= u16::max_value() as usize {
            Err(Error::StringTooLong)
        } else {
            let len = v.len() as u16;
            self.incr_length(len)?;
            // note that this is not a mistake
            // for some reasons Tezos encodes strings length as u32
            self.serialize_u16(0)?;
            self.serialize_u16(len)?;
            io::Write::write(&mut self.output, v.as_bytes())?;
            Ok(())
        }
    }
    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        // FIXME should be the opposite
        self.serialize_bytes(if v { &[0] } else { &[1] })
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        io::Write::write(&mut self.output, &[(v >> 8 & 0xff) as u8, (v & 0xff) as u8])?;
        self.incr_length(2)
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        let len = v.len() as u16;
        self.incr_length(len)?;
        io::Write::write(&mut self.output, v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(self)
    }
}
impl<'a> SerializeSeq for &'a mut TezosBinSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> SerializeTuple for &'a mut TezosBinSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> SerializeTupleStruct for &'a mut TezosBinSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> SerializeTupleVariant for &'a mut TezosBinSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> SerializeMap for &'a mut TezosBinSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }
    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }
    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, _key: &K, _value: &V) -> Result<()>
    where
        K: serde::Serialize,
        V: serde::Serialize,
    {
        unimplemented!()
    }
    fn end(self) -> Result<Self::Ok> {
        unimplemented!()
    }
}

impl<'a> SerializeStruct for &'a mut TezosBinSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }
    fn skip_field(&mut self, _key: &'static str) -> Result<()> {
        Ok(())
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> SerializeStructVariant for &'a mut TezosBinSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }
    fn skip_field(&mut self, _key: &'static str) -> Result<()> {
        Ok(())
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

/// `serde::de::Visitor` for fixed sized byte buffers.
pub struct BuffVisitor<const N: usize>;

impl<'de, const N: usize> Visitor<'de> for BuffVisitor<N> {
    type Value = [u8; N];
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_fmt(format_args!("expected an array of bytes of size {}", N))
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut bytes = [0; N];
        for i in 0..N {
            if let Ok(Some(b)) = seq.next_element::<u8>() {
                bytes[i] = b;
            } else {
                return Err(de::Error::invalid_length(i, &self));
            }
        }
        Ok(bytes)
    }
}

/// Deserializes a structure from a byte array
pub fn from_bytes<T>(input: &mut [u8]) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let mut deserializer = TezosBinDeserializer::from_bytes(input);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.is_empty() {
        Ok(t)
    } else {
        Err(Error::ExtraBytes)
    }
}

struct TezosBinDeserializer<'de> {
    input: &'de [u8],
}

impl<'de> TezosBinDeserializer<'de> {
    fn from_bytes(input: &'de [u8]) -> Self {
        TezosBinDeserializer { input }
    }
    fn is_empty(&self) -> bool {
        self.input.len() == 0
    }
    fn next(&mut self) -> Option<u8> {
        if self.input.len() == 0 {
            None
        } else {
            let elt = self.input[0];
            self.input = &self.input[1..];
            Some(elt)
        }
    }

    fn read_u16(&mut self) -> Result<u16> {
        if self.input.len() < 2 {
            Err(Error::UnsufficentBytes)
        } else {
            let mut v = (self.input[0] as u16) << 8;
            v |= self.input[1] as u16;
            self.input = &self.input[2..];
            Ok(v)
        }
    }

    fn read_bool(&mut self) -> Result<bool> {
        if self.input.len() < 1 {
            Err(Error::UnsufficentBytes)
        } else {
            let b = self.input[0] == 0; // FIXME should be a 1 but this is a hack just for ack
            self.input = &self.input[1..];
            Ok(b)
        }
    }

    fn ensure_bytes(&mut self, size: usize) -> Result<Vec<u8>> {
        if self.input.len() < size {
            Err(Error::UnsufficentBytes)
        } else {
            let mut buff = vec![0; size];
            buff.copy_from_slice(&self.input[..size]);
            self.input = &self.input[size..];
            Ok(buff)
        }
    }
}

impl<'de, 'a> Deserializer<'de> for &'a mut TezosBinDeserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        i8 i16 i32 i64 u32 u64 f32 f64 char str bytes byte_buf option unit
        unit_struct tuple tuple_struct map enum identifier ignored_any
    }
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let _ = visitor;
        unimplemented!(
            r#"If you'd like to further enhance this protocol consider first implementing a more performant one similar to binary_serde but with support of extra self describing data (such as variable sized strings)"#
        )
    }

    /// Quite inefficient!!
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let b = self.next().ok_or_else(|| Error::UnsufficentBytes)?;
        visitor.visit_u8(b)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_u16()?;
        visitor.visit_u16(v)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let zeros = self.read_u16()?;
        assert_eq!(
            0, zeros,
            "the size of strings should not be larger than a full packet"
        );
        let size = self.read_u16()?;
        let buff = self.ensure_bytes(size as usize)?;
        let s = String::from_utf8(buff)?;
        visitor.visit_string(s)
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let _ = name;
        visitor.visit_seq(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
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
        let _ = name;
        let _ = fields;
        visitor.visit_seq(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let b = self.read_bool()?;
        visitor.visit_bool(b)
    }
}

impl<'de> SeqAccess<'de> for TezosBinDeserializer<'de> {
    type Error = Error;

    fn next_element_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>>
    where
        S: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(Some)
    }
}

/// Generic module for binary serialization of messages exchanged in the Tezos p2p protocol.
/// Note: currently partially implemented to fullfill the handshake.
use std::io;

use serde::{
    de::{self, Visitor},
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize, Serializer,
};

use super::error::{Error, Result};

/// Serializes plain data in the packed binary format of Tezos P2P protocol, the data is fully returned
/// and ready to be sent to peers as is. The first 2 bytes will contain the header which is the size of
/// the message encoded as a `u16`.
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
        todo!()
    }
    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        todo!()
    }
    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, _key: &K, _value: &V) -> Result<()>
    where
        K: serde::Serialize,
        V: serde::Serialize,
    {
        todo!()
    }
    fn end(self) -> Result<Self::Ok> {
        todo!()
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
            // note that this is not a mistake
            // for some reasons Tezos encodes strings length as u32
            self.incr_length(v.len() as u16)?;
            self.serialize_u32(v.len() as u32)?;
            io::Write::write(&mut self.output, v.as_bytes())?;
            Ok(())
        }
    }
    fn serialize_bool(self, _v: bool) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        io::Write::write(&mut self.output, &[(v >> 8 & 0xff) as u8, (v & 0xff) as u8])?;
        self.incr_length(2)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        io::Write::write(
            &mut self.output,
            &[0, 0, (v >> 8 & 0xff) as u8, (v & 0xff) as u8],
        )?;
        self.incr_length(4)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        let len = v.len() as u16;
        self.incr_length(2 + len)?;

        io::Write::write(&mut self.output, &[(len >> 8) as u8, (len & 0xff) as u8])?;
        io::Write::write(&mut self.output, v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        todo!()
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
        todo!()
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

/// `serde::de::Visitor` for fixed sized byte buffers.
pub struct BuffVisitor<const N: usize>;

impl<'de, const N: usize> Visitor<'de> for BuffVisitor<N> {
    type Value = [u8; N];
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_fmt(format_args!("expected an array of bytes of size {}", N))
    }
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v.len() < N {
            Err(de::Error::invalid_length(v.len(), &self))
        } else {
            let mut output = [0u8; N];
            output.copy_from_slice(&v[..N]);
            Ok(output)
        }
    }
}

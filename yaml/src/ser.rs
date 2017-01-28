// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! YAML Serialization
//!
//! This module provides YAML serialization with the type `Serializer`.

use std::{fmt, io};

use yaml_rust::{yaml, Yaml, YamlEmitter};

use serde::ser;

use super::error::{Error, Result};

pub struct Serializer;

impl ser::Serializer for Serializer {
    type Ok = Yaml;
    type Error = Error;

    type SerializeSeq = SerializeArray;
    type SerializeTuple = SerializeArray;
    type SerializeTupleStruct = SerializeArray;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Yaml> {
        Ok(Yaml::Boolean(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Yaml> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Yaml> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Yaml> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Yaml> {
        Ok(Yaml::Integer(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Yaml> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u16(self, v: u16) -> Result<Yaml> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u32(self, v: u32) -> Result<Yaml> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u64(self, v: u64) -> Result<Yaml> {
        self.serialize_i64(v as i64)
    }

    fn serialize_f32(self, v: f32) -> Result<Yaml> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Yaml> {
        Ok(Yaml::Real(v.to_string()))
    }

    fn serialize_char(self, value: char) -> Result<Yaml> {
        Ok(Yaml::String(value.to_string()))
    }

    fn serialize_str(self, value: &str) -> Result<Yaml> {
        Ok(Yaml::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Yaml> {
        let vec = value.iter().map(|&b| Yaml::Integer(b as i64)).collect();
        Ok(Yaml::Array(vec))
    }

    fn serialize_unit(self) -> Result<Yaml> {
        Ok(Yaml::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Yaml> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &str,
        _variant_index: usize,
        variant: &str
    ) -> Result<Yaml> {
        Ok(Yaml::String(variant.to_owned()))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T
    ) -> Result<Yaml>
        where T: ser::Serialize
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &str,
        _variant_index: usize,
        variant: &str,
        value: &T
    ) -> Result<Yaml>
        where T: ser::Serialize
    {
        Ok(singleton_hash(try!(to_yaml(variant)), try!(to_yaml(value))))
    }

    fn serialize_none(self) -> Result<Yaml> {
        self.serialize_unit()
    }

    fn serialize_some<V: ?Sized>(self, value: &V) -> Result<Yaml>
        where V: ser::Serialize
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SerializeArray> {
        let array = match len {
            None => yaml::Array::new(),
            Some(len) => yaml::Array::with_capacity(len),
        };
        Ok(SerializeArray { array: array })
    }

    fn serialize_seq_fixed_size(self, len: usize) -> Result<SerializeArray> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple(self, len: usize) -> Result<SerializeArray> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize
    ) -> Result<SerializeArray> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _enum: &'static str,
        _idx: usize,
        variant: &'static str,
        len: usize
    ) -> Result<SerializeTupleVariant> {
        Ok(SerializeTupleVariant { name: variant, array: yaml::Array::with_capacity(len) })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<SerializeMap> {
        Ok(SerializeMap { hash: yaml::Hash::new(), next_key: None })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize
    ) -> Result<SerializeStruct> {
        Ok(SerializeStruct { hash: yaml::Hash::new() })
    }

    fn serialize_struct_variant(
        self,
        _enum: &'static str,
        _idx: usize,
        variant: &'static str,
        _len: usize
    ) -> Result<SerializeStructVariant> {
        Ok(SerializeStructVariant { name: variant, hash: yaml::Hash::new() })
    }
}

#[doc(hidden)]
pub struct SerializeArray {
    array: yaml::Array,
}

#[doc(hidden)]
pub struct SerializeTupleVariant {
    name: &'static str,
    array: yaml::Array,
}

#[doc(hidden)]
pub struct SerializeMap {
    hash: yaml::Hash,
    next_key: Option<yaml::Yaml>,
}

#[doc(hidden)]
pub struct SerializeStruct {
    hash: yaml::Hash,
}

#[doc(hidden)]
pub struct SerializeStructVariant {
    name: &'static str,
    hash: yaml::Hash,
}

impl ser::SerializeSeq for SerializeArray {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, elem: &T) -> Result<()>
        where T: ser::Serialize
    {
        self.array.push(try!(to_yaml(elem)));
        Ok(())
    }

    fn end(self) -> Result<Yaml> {
        Ok(Yaml::Array(self.array))
    }
}

impl ser::SerializeTuple for SerializeArray {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, elem: &T) -> Result<()>
        where T: ser::Serialize
    {
        ser::SerializeSeq::serialize_element(self, elem)
    }

    fn end(self) -> Result<Yaml> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SerializeArray {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_field<V: ?Sized>(&mut self, value: &V) -> Result<()>
        where V: ser::Serialize
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Yaml> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_field<V: ?Sized>(&mut self, v: &V) -> Result<()>
        where V: ser::Serialize
    {
        self.array.push(try!(to_yaml(v)));
        Ok(())
    }

    fn end(self) -> Result<Yaml> {
        Ok(singleton_hash(try!(to_yaml(self.name)), Yaml::Array(self.array)))
    }
}

impl ser::SerializeMap for SerializeMap {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
        where T: ser::Serialize
    {
        self.next_key = Some(try!(to_yaml(key)));
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        match self.next_key.take() {
            Some(key) => self.hash.insert(key, try!(to_yaml(value))),
            None => panic!("serialize_value called before serialize_key"),
        };
        Ok(())
    }

    fn end(self) -> Result<Yaml> {
        Ok(Yaml::Hash(self.hash))
    }
}

impl ser::SerializeStruct for SerializeStruct {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_field<V: ?Sized>(&mut self, key: &'static str, value: &V) -> Result<()>
        where V: ser::Serialize
    {
        self.hash.insert(try!(to_yaml(key)), try!(to_yaml(value)));
        Ok(())
    }

    fn end(self) -> Result<Yaml> {
        Ok(Yaml::Hash(self.hash))
    }
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_field<V: ?Sized>(&mut self, field: &'static str, v: &V) -> Result<()>
        where V: ser::Serialize
    {
        self.hash.insert(try!(to_yaml(field)), try!(to_yaml(v)));
        Ok(())
    }

    fn end(self) -> Result<Yaml> {
        Ok(singleton_hash(try!(to_yaml(self.name)), Yaml::Hash(self.hash)))
    }
}

pub fn to_writer<W, T>(writer: &mut W, value: &T) -> Result<()>
    where W: io::Write,
          T: ser::Serialize
{
    let doc = try!(to_yaml(value));
    let mut writer_adapter = FmtToIoWriter {
        writer: writer,
    };
    try!(YamlEmitter::new(&mut writer_adapter).dump(&doc));
    Ok(())
}

pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
    where T: ser::Serialize
{
    let mut vec = Vec::with_capacity(128);
    try!(to_writer(&mut vec, value));
    Ok(vec)
}

pub fn to_string<T>(value: &T) -> Result<String>
    where T: ser::Serialize
{
    Ok(try!(String::from_utf8(try!(to_vec(value)))))
}

/// The yaml-rust library uses `fmt.Write` intead of `io.Write` so this is a
/// simple adapter.
struct FmtToIoWriter<'a, W>
    where W: io::Write + 'a
{
    writer: &'a mut W,
}

impl<'a, W> fmt::Write for FmtToIoWriter<'a, W>
    where W: io::Write + 'a
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.writer.write(s.as_bytes()).is_err() {
            return Err(fmt::Error);
        }
        Ok(())
    }
}

fn to_yaml<T>(elem: T) -> Result<Yaml>
    where T: ser::Serialize
{
    elem.serialize(Serializer)
}

fn singleton_hash(k: Yaml, v: Yaml) -> Yaml {
    let mut hash = yaml::Hash::new();
    hash.insert(k, v);
    Yaml::Hash(hash)
}
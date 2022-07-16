// this file is pretty much entirely shamelessly stolen from serde_json
// https://github.com/serde-rs/json/blob/dab5ed3ee97cef5e2b796513f8d9e4c7416e44bf/src/value/de.rs

use core::slice;
use std::collections::HashMap;

use awlyc_error::Diagnostic;
use serde::{
    de::{
        self, value::StringDeserializer, DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess,
        Unexpected,
    },
    Deserializer,
};
use smol_str::SmolStr;

use crate::AwlycValue;

pub fn from_awlyc_val<T>(value: &AwlycValue) -> Result<T, Diagnostic>
where
    T: DeserializeOwned,
{
    T::deserialize(value)
}

impl<'de, 'a: 'de> Deserializer<'de> for &'a AwlycValue {
    type Error = Diagnostic;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match &self {
            AwlycValue::Null => visitor.visit_unit(),
            AwlycValue::Int(v) => visitor.visit_i64(*v),
            AwlycValue::Float(v) => visitor.visit_f64(*v),
            AwlycValue::String(v) => visitor.visit_str(v),
            AwlycValue::Record(v) => visit_object_ref(v, visitor),
            AwlycValue::Array(v) => visit_array_ref(v, visitor),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            AwlycValue::Array(v) => visit_array_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            AwlycValue::Array(v) => visit_array_ref(v, visitor),
            AwlycValue::Record(v) => visit_object_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            AwlycValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    serde::forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string
        bytes byte_buf unit map newtype_struct enum
        ignored_any unit_struct tuple_struct tuple identifier
    }
}

impl AwlycValue {
    fn invalid_type<E>(&self, exp: &dyn de::Expected) -> E
    where
        E: serde::de::Error,
    {
        serde::de::Error::invalid_type(self.unexpected(), exp)
    }

    fn unexpected(&self) -> Unexpected {
        match self {
            AwlycValue::Null => Unexpected::Unit,
            AwlycValue::Int(_) => Unexpected::Signed(64),
            AwlycValue::Float(_) => Unexpected::Float(64.0),
            AwlycValue::String(v) => Unexpected::Str(v),
            AwlycValue::Array(_) => Unexpected::Seq,
            AwlycValue::Record(_) => Unexpected::Map,
        }
    }
}

fn visit_array_ref<'de, V>(array: &'de [AwlycValue], visitor: V) -> Result<V::Value, Diagnostic>
where
    V: de::Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqRefDeserializer::new(array);
    let seq = match visitor.visit_seq(&mut deserializer) {
        core::result::Result::Ok(v) => v,
        core::result::Result::Err(err) => return core::result::Result::Err(err),
    };
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in array",
        ))
    }
}

fn visit_object_ref<'de, V>(
    object: &'de HashMap<SmolStr, AwlycValue>,
    visitor: V,
) -> Result<V::Value, Diagnostic>
where
    V: de::Visitor<'de>,
{
    let len = object.len();
    let mut deserializer = MapRefDeserializer::new(object);
    let map = match visitor.visit_map(&mut deserializer) {
        core::result::Result::Ok(v) => v,
        core::result::Result::Err(err) => return core::result::Result::Err(err),
    };
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}

struct SeqRefDeserializer<'de> {
    iter: slice::Iter<'de, AwlycValue>,
}

impl<'de> SeqRefDeserializer<'de> {
    fn new(slice: &'de [AwlycValue]) -> Self {
        SeqRefDeserializer { iter: slice.iter() }
    }
}

impl<'de> SeqAccess<'de> for SeqRefDeserializer<'de> {
    type Error = Diagnostic;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Diagnostic>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct MapRefDeserializer<'de> {
    iter: <&'de HashMap<SmolStr, AwlycValue> as IntoIterator>::IntoIter,
    value: Option<&'de AwlycValue>,
}

impl<'de> MapRefDeserializer<'de> {
    fn new(map: &'de HashMap<SmolStr, AwlycValue>) -> Self {
        MapRefDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapRefDeserializer<'de> {
    type Error = Diagnostic;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Diagnostic>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(StringDeserializer::new(key.to_string()))
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Diagnostic>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

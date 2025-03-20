use alloy_primitives::{Address, Bytes, FixedBytes, aliases::*, hex};
use std::any::Any;
use std::fmt::Debug;

pub trait BoxTrait: Any + Debug + EncodeCodec {
    fn encode_codec(&self) -> &dyn EncodeCodec;
    fn clone_box(&self) -> Box<dyn BoxTrait>;
}

impl<T: Any + EncodeCodec + Debug + Clone + 'static> BoxTrait for T {
    fn encode_codec(&self) -> &dyn EncodeCodec {
        self
    }

    fn clone_box(&self) -> Box<dyn BoxTrait> {
        Box::new(self.clone())
    }
}

pub trait EncodeCodec: Any {
    fn to_bytes_vec(&self) -> Vec<u8>;
    fn bytes_length(&self) -> usize;
    fn eth_type(&self) -> String;
    fn to_string(&self) -> String;
    fn as_any(&self) -> &dyn Any;
}

// Separate trait for construction
pub trait DecodeCodec {
    fn from_bytes<const BYTES: usize>(bytes: [u8; BYTES]) -> Self;
}

impl<const N: usize> EncodeCodec for FixedBytes<N> {
    fn to_bytes_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }

    fn bytes_length(&self) -> usize {
        Self::len_bytes()
    }

    fn eth_type(&self) -> String {
        format!("bytes{}", self.bytes_length())
    }

    fn to_string(&self) -> String {
        hex::encode(self.as_slice())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<const N: usize> DecodeCodec for FixedBytes<N> {
    fn from_bytes<const BYTES: usize>(bytes: [u8; BYTES]) -> Self {
        Self::from_slice(&bytes)
    }
}

impl EncodeCodec for String {
    fn to_bytes_vec(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn bytes_length(&self) -> usize {
        self.len()
    }

    fn eth_type(&self) -> String {
        "string".to_string()
    }

    fn to_string(&self) -> String {
        self.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl DecodeCodec for String {
    fn from_bytes<const BYTES: usize>(bytes: [u8; BYTES]) -> Self {
        String::from_utf8(bytes.to_vec()).unwrap()
    }
}

impl EncodeCodec for Address {
    fn to_bytes_vec(&self) -> Vec<u8> {
        self.into_array().to_vec()
    }

    fn bytes_length(&self) -> usize {
        Address::len_bytes()
    }

    fn eth_type(&self) -> String {
        "address".to_string()
    }

    fn to_string(&self) -> String {
        format!("{}", self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl DecodeCodec for Address {
    fn from_bytes<const BYTES: usize>(bytes: [u8; BYTES]) -> Self {
        Address::from_slice(&bytes)
    }
}

impl EncodeCodec for Bytes {
    fn to_bytes_vec(&self) -> Vec<u8> {
        self.to_vec()
    }

    fn bytes_length(&self) -> usize {
        self.len()
    }

    fn eth_type(&self) -> String {
        "bytes".to_string()
    }

    fn to_string(&self) -> String {
        format!("{}", self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl DecodeCodec for Bytes {
    fn from_bytes<const BYTES: usize>(bytes: [u8; BYTES]) -> Self {
        Bytes::copy_from_slice(&bytes)
    }
}

impl EncodeCodec for bool {
    fn to_bytes_vec(&self) -> Vec<u8> {
        vec![*self as u8]
    }

    fn bytes_length(&self) -> usize {
        1
    }

    fn eth_type(&self) -> String {
        "bool".to_string()
    }

    fn to_string(&self) -> String {
        format!("{}", self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl DecodeCodec for bool {
    fn from_bytes<const BYTES: usize>(bytes: [u8; BYTES]) -> Self {
        bytes[0] != 0
    }
}

impl EncodeCodec for Value {
    fn to_bytes_vec(&self) -> Vec<u8> {
        match self {
            Value::Single(value, _) => value.to_bytes_vec(),
            Value::Collection(values) => {
                values.iter().map(|v| v.to_bytes_vec()).flatten().collect()
            }
        }
    }

    fn bytes_length(&self) -> usize {
        match self {
            Value::Single(value, _) => value.bytes_length(),
            Value::Collection(values) => values.iter().map(|v| v.bytes_length()).sum(),
        }
    }

    fn eth_type(&self) -> String {
        match self {
            Value::Single(_, type_of) => type_of.clone(),
            Value::Collection(values) => values
                .iter()
                .map(|v| v.eth_type())
                .collect::<Vec<String>>()
                .join(","),
        }
    }

    fn to_string(&self) -> String {
        match self {
            Value::Single(value, _) => value.to_string(),
            Value::Collection(values) => values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(", "),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<T: EncodeCodec> EncodeCodec for Vec<T> {
    fn to_bytes_vec(&self) -> Vec<u8> {
        Vec::new()
    }

    fn bytes_length(&self) -> usize {
        0
    }

    fn eth_type(&self) -> String {
        String::new()
    }

    fn to_string(&self) -> String {
        String::new()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn pad_left(input: Vec<u8>, target_length: usize) -> Vec<u8> {
    if input.len() >= target_length {
        return input;
    }

    let padding_size = target_length - input.len();
    let mut padded = vec![0; padding_size];
    padded.extend(input);
    padded
}

pub fn pad_right(input: Vec<u8>, target_length: usize) -> Vec<u8> {
    if input.len() >= target_length {
        return input;
    }

    let mut padded = input;
    padded.resize(target_length, 0);
    padded
}

// Add this function to codec.rs

impl Clone for Box<dyn BoxTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl EncodeCodec for Vec<Box<dyn BoxTrait>> {
    fn to_bytes_vec(&self) -> Vec<u8> {
        self.iter().flat_map(|v| v.to_bytes_vec()).collect()
    }

    fn bytes_length(&self) -> usize {
        self.iter().map(|v| v.bytes_length()).sum()
    }

    fn eth_type(&self) -> String {
        if let Some(first) = self.first() {
            format!("{}[]", first.eth_type())
        } else {
            String::new()
        }
    }

    fn to_string(&self) -> String {
        self.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct ValueBuilder {
    values: Vec<Value>,
}

impl ValueBuilder {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn add<T: BoxTrait + 'static>(&mut self, value: T) -> &mut Self {
        let eth_type = value.eth_type();
        self.values.push(create_value(value, eth_type.as_str()));
        self
    }

    pub fn add_array<T: BoxTrait + 'static>(&mut self, values: Vec<T>) -> &mut Self {
        let inner_values: Vec<Value> = if let Some(first) = values.first() {
            if first.as_any().is::<Vec<Box<dyn BoxTrait>>>() {
                // Handle array of tuples
                values
                    .into_iter()
                    .map(|v| {
                        if let Some(tuple) = v.as_any().downcast_ref::<Vec<Box<dyn BoxTrait>>>() {
                            let mut tuple_values = Vec::new();
                            for value in tuple {
                                if let Some(string_vec) =
                                    value.encode_codec().as_any().downcast_ref::<Vec<String>>()
                                {
                                    let string_values: Vec<Value> = string_vec
                                        .iter()
                                        .map(|s| {
                                            Value::Single(
                                                Box::new(s.clone()) as Box<dyn BoxTrait>,
                                                "string".to_string(),
                                            )
                                        })
                                        .collect();
                                    tuple_values.push(Value::Collection(string_values));
                                } else {
                                    let type_str = value.eth_type();
                                    tuple_values.push(Value::Single(value.clone_box(), type_str));
                                }
                            }
                            Value::Collection(tuple_values)
                        } else {
                            panic!("Expected tuple value")
                        }
                    })
                    .collect()
            } else {
                let type_str = values.first().unwrap().eth_type();
                // Handle regular arrays
                values
                    .into_iter()
                    .map(|v| Value::Single(Box::new(v) as Box<dyn BoxTrait>, type_str.clone()))
                    .collect()
            }
        } else {
            Vec::new()
        };

        self.values.push(Value::Collection(inner_values));
        self
    }

    pub fn add_tuple(&mut self, values: Vec<Box<dyn BoxTrait>>) -> &mut Self {
        let mut inner_values = Vec::new();
        for value in values {
            if let Some(string_vec) = value.as_any().downcast_ref::<Vec<String>>() {
                let string_values: Vec<Value> = string_vec
                    .iter()
                    .map(|s| {
                        Value::Single(
                            Box::new(s.clone()) as Box<dyn BoxTrait>,
                            "string".to_string(),
                        )
                    })
                    .collect();
                inner_values.push(Value::Collection(string_values));
            } else {
                let type_str = value.eth_type();
                inner_values.push(Value::Single(value, type_str));
            }
        }

        self.values.push(Value::Collection(inner_values));
        self
    }

    pub fn build(&self) -> Vec<Value> {
        self.values.clone()
    }
}

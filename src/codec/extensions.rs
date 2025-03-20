use crate::codec::traits::{DecodeCodec, EncodeCodec};
use alloy_primitives::{Address, Bytes, FixedBytes, hex};
use std::any::Any;

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

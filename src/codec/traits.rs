use std::any::Any;
use std::fmt::Debug;

pub trait BoxTrait: Any + Debug + EncodeCodec {
    fn encode_codec(&self) -> &dyn EncodeCodec;
    fn clone_box(&self) -> Box<dyn BoxTrait>;
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

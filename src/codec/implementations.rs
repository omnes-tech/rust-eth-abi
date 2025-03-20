use crate::codec::traits::BoxTrait;
use crate::codec::traits::EncodeCodec;
use crate::codec::types::Value;
use std::any::Any;
use std::fmt::Debug;

impl<T: Any + EncodeCodec + Debug + Clone + 'static> BoxTrait for T {
    fn encode_codec(&self) -> &dyn EncodeCodec {
        self
    }

    fn clone_box(&self) -> Box<dyn BoxTrait> {
        Box::new(self.clone())
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

impl Clone for Box<dyn BoxTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

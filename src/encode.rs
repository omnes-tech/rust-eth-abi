use crate::codec::traits::EncodeCodec;
use crate::codec::types::Value;
use crate::codec::utils::{get_collection_i, pad_left, pad_right};
use crate::common::{check_type_and_value, is_array, is_dynamic, is_tuple, split_parameter_types};
use crate::errors::CodecError;
use alloy_primitives::aliases::U256;
use alloy_primitives::utils::keccak256;

#[derive(Debug)]
struct DynamicPlaceholder {
    header_offset: usize,
    footer_offset: usize,
}

pub fn abi_encode_with_selector(
    selector: &[u8; 4],
    type_strs: &Vec<&str>,
    values: &Vec<Value>,
) -> Result<Vec<u8>, CodecError> {
    let encoded = abi_encode(&type_strs, values)?;

    Ok(selector
        .to_vec()
        .into_iter()
        .chain(encoded.into_iter())
        .collect())
}

pub fn abi_encode_selector(signature: &str) -> Result<Vec<u8>, CodecError> {
    let selector = keccak256(signature.as_bytes());

    Ok(selector.to_bytes_vec()[0..4].to_vec())
}

pub fn abi_encode_with_singature(
    signature: &str,
    values: &Vec<Value>,
) -> Result<Vec<u8>, CodecError> {
    let selector = abi_encode_selector(signature)?;
    let type_strs = split_parameter_types(signature);
    let encoded = abi_encode(&type_strs, values)?;

    Ok(selector.into_iter().chain(encoded.into_iter()).collect())
}

pub fn abi_encode_packed(
    type_strs: &Vec<&str>,
    values: &Vec<Value>,
) -> Result<Vec<u8>, CodecError> {
    if type_strs.len() != values.len() {
        return Err(CodecError::LengthsMismatch(type_strs.len(), values.len()));
    }

    let mut encoded = Vec::new();
    for (i, (type_str, value)) in type_strs.iter().zip(values.iter()).enumerate() {
        let (is_array_type, _) = is_array(type_str)?;
        let (is_tuple_type, tuple_types) = is_tuple(type_str)?;

        let encoded_value = if is_array_type {
            let value = get_collection_i(values, i);
            encode_packed_array(type_str, &value)?
        } else if is_tuple_type {
            let value = get_collection_i(values, i);
            abi_encode_packed(&tuple_types, &value)?
        } else {
            encode_packed(type_str, value)?
        };

        encoded.extend(encoded_value);
    }

    Ok(encoded)
}

pub fn abi_encode(type_strs: &Vec<&str>, values: &Vec<Value>) -> Result<Vec<u8>, CodecError> {
    if type_strs.len() != values.len() {
        return Err(CodecError::LengthsMismatch(type_strs.len(), values.len()));
    }

    let mut header: Vec<u8> = Vec::new();
    let mut dyn_header_placeholder: Vec<DynamicPlaceholder> = Vec::new();
    let mut footer: Vec<u8> = Vec::new();
    for (i, (type_str, value)) in type_strs.iter().zip(values.iter()).enumerate() {
        let (is_array_type, size) = is_array(type_str)?;
        let is_dynamic_type = is_dynamic(type_str);
        let (is_tuple_type, tuple_types) = is_tuple(type_str)?;

        let encoded_value = if is_array_type {
            let value = get_collection_i(values, i);
            encode_array(
                type_str,
                &value,
                size,
                is_dynamic_type,
                is_tuple_type,
                &tuple_types,
            )?
        } else if is_tuple_type {
            let value = get_collection_i(values, i);
            abi_encode(&tuple_types, &value)?
        } else {
            encode(type_str, value, is_dynamic_type)?
        };

        if is_dynamic_type {
            let placeholder = pad_right(Vec::new(), 32);
            dyn_header_placeholder.push(DynamicPlaceholder {
                header_offset: i * 32,
                footer_offset: footer.len(),
            });

            footer.extend(encoded_value);
            header.extend(placeholder);
        } else {
            let encoded_value = encode(type_str, value, is_dynamic_type)?;
            header.extend(encoded_value);
        };
    }

    for placeholder in dyn_header_placeholder {
        let offset = placeholder.header_offset;
        let value = U256::from(header.len() + placeholder.footer_offset);
        header[offset..offset + value.bytes_length()].copy_from_slice(&value.to_bytes_vec());
    }
    header.extend(footer);

    Ok(header)
}

fn encode(type_str: &str, value: &Value, is_dynamic_type: bool) -> Result<Vec<u8>, CodecError> {
    let mut encoded = encode_packed(type_str, value)?;

    if is_dynamic_type {
        let length = encoded.len();
        encoded = pad_right(encoded, 32);
        let length = U256::from(length);
        encoded = length
            .to_bytes_vec()
            .into_iter()
            .chain(encoded.into_iter())
            .collect();
    } else {
        encoded = pad_left(encoded, 32);
    }

    Ok(encoded)
}

fn encode_array(
    arr_type_str: &str,
    values: &Vec<Value>,
    size: usize,
    is_dynamic_type: bool,
    is_tuple_type: bool,
    tuple_types: &Vec<&str>,
) -> Result<Vec<u8>, CodecError> {
    if size != 0 && size != values.len() {
        return Err(CodecError::InvalidTypeAndValue(
            arr_type_str.to_string(),
            format!(
                "type array length != value array length: {} != {}",
                size,
                values.len()
            ),
        ));
    }
    let type_str = arr_type_str.split("[").next().unwrap();

    let mut header: Vec<u8> = Vec::new();

    let mut dyn_header_placeholder: Vec<DynamicPlaceholder> = Vec::new();
    let mut footer: Vec<u8> = Vec::new();
    for (i, value) in values.iter().enumerate() {
        let encoded_value = if is_tuple_type {
            let value = get_collection_i(values, i);
            abi_encode(tuple_types, &value)?
        } else {
            encode(type_str, value, is_dynamic_type)?
        };

        if is_dynamic_type {
            let placeholder = pad_right(Vec::new(), 32);
            dyn_header_placeholder.push(DynamicPlaceholder {
                header_offset: i * 32,
                footer_offset: footer.len(),
            });

            footer.extend(encoded_value);
            header.extend(placeholder);
        } else {
            let encoded_value = encode(type_str, value, is_dynamic_type)?;
            header.extend(encoded_value);
        };
    }

    for placeholder in dyn_header_placeholder {
        let offset = placeholder.header_offset;
        let value = U256::from(header.len() + placeholder.footer_offset);
        header[offset..offset + value.bytes_length()].copy_from_slice(&value.to_bytes_vec());
    }
    header.extend(footer);

    if size == 0 {
        let array_length = U256::from(values.len());
        header = array_length
            .to_bytes_vec()
            .into_iter()
            .chain(header.into_iter())
            .collect();
    }

    Ok(header)
}

fn encode_packed(type_str: &str, value: &Value) -> Result<Vec<u8>, CodecError> {
    if !check_type_and_value(type_str, value) {
        return Err(CodecError::InvalidTypeAndValue(
            type_str.to_string(),
            value.to_string(),
        ));
    }

    Ok(value.to_bytes_vec())
}

fn encode_packed_array(type_str: &str, values: &Vec<Value>) -> Result<Vec<u8>, CodecError> {
    let mut encoded = Vec::new();
    for value in values {
        let encoded_value = encode_packed(type_str, value)?;
        encoded = encoded_value
            .into_iter()
            .chain(encoded.into_iter())
            .collect();
    }

    Ok(encoded)
}

#[cfg(test)]
mod encode_tests {
    use super::*;
    use crate::build_values;
    use crate::codec::traits::BoxTrait;
    use crate::codec::types::ValueBuilder;
    use alloy_primitives::{Address, aliases::*, hex};

    #[test]
    fn test_abi_encode_regular() {
        let type_strs = vec!["uint256", "uint256", "address"];
        let values = build_values![
            Box::new(U256::from(1)) as Box<dyn BoxTrait>,
            Box::new(U256::from(2)) as Box<dyn BoxTrait>,
            Box::new(Address::ZERO) as Box<dyn BoxTrait>
        ];

        let encoded = abi_encode(&type_strs, &values).unwrap();
        assert_eq!(
            hex::encode(&encoded),
            "000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_abi_encode_array() {
        let type_strs = vec!["address", "string[2]", "uint256"];
        let values = ValueBuilder::new()
            .add(Address::ZERO)
            .add_array(vec![
                String::from("Hello, world!"),
                String::from("Hello, world!"),
            ])
            .add(U256::from(1))
            .build();
        println!("values: {:?}", values);
        let encoded = abi_encode(&type_strs, &values).unwrap();
        assert_eq!(
            hex::encode(&encoded),
            "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_abi_encode_tuple() {
        let type_strs = vec!["address", "(string[],uint256,uint8)", "uint256"];
        let values = ValueBuilder::new()
            .add(Address::ZERO)
            .add_tuple(vec![
                Box::new(vec![
                    String::from("Hello, world!"),
                    String::from("Hello, world!"),
                ]),
                Box::new(U256::from(1)) as Box<dyn BoxTrait>,
                Box::new(U8::from(1)) as Box<dyn BoxTrait>,
            ])
            .add(U256::from(1))
            .build();
        println!("values: {:?}", values);
        let encoded = abi_encode(&type_strs, &values).unwrap();
        assert_eq!(
            hex::encode(&encoded),
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_abi_encode_tuple_array() {
        let type_strs = vec!["address", "(string[],uint256,uint8)[]", "uint256"];
        let values = ValueBuilder::new()
            .add(Address::ZERO)
            .add_array(vec![vec![
                Box::new(vec![
                    String::from("Hello, world!"),
                    String::from("Hello, world!"),
                ]) as Box<dyn BoxTrait>,
                Box::new(U256::from(1)) as Box<dyn BoxTrait>,
                Box::new(U8::from(1)) as Box<dyn BoxTrait>,
            ]]) // Now this should work for both regular arrays and arrays of tuples
            .add(U256::from(1))
            .build();

        let encoded = abi_encode(&type_strs, &values).unwrap();
        assert_eq!(
            hex::encode(&encoded),
            "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_encode_packed() {
        let value = build_values!(Box::new(U256::from(1)) as Box<dyn BoxTrait>);
        let encoded = encode_packed("uint256", &value).unwrap();
        assert_eq!(hex::encode(&encoded), hex::encode(value.to_bytes_vec()));
    }

    #[test]
    fn test_encode() {
        let value = build_values!(Box::new(String::from("Hello, world!")) as Box<dyn BoxTrait>);
        let encoded = encode("string", &value, true).unwrap();
        assert_eq!(
            hex::encode(&encoded),
            "000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000"
        );
    }
}

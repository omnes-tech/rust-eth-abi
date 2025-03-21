use crate::codec::traits::DecodeCodec;
use crate::codec::types::Value;
use crate::common::{get_bytes_from_type, is_array, is_dynamic, is_tuple, split_parameter_types};
use crate::encode::abi_encode_selector;
use crate::errors::CodecError;
use alloy_primitives::{Address, Bytes, FixedBytes, aliases::*};

pub fn abi_decode_with_signature(
    signature: &str,
    encoded_values: &Vec<u8>,
) -> Result<Vec<Value>, CodecError> {
    let selector = abi_encode_selector(signature)?;
    let type_strs = split_parameter_types(signature);
    if selector != encoded_values[..4] {
        return Err(CodecError::InvalidSelector);
    }

    let encoded_values = &encoded_values[4..];

    abi_decode(&type_strs, &encoded_values.to_vec())
}

pub fn abi_decode(
    type_strs: &Vec<&str>,
    encoded_values: &Vec<u8>,
) -> Result<Vec<Value>, CodecError> {
    let mut cursor = 0;
    let mut values = Vec::new();

    for type_str in type_strs {
        let (is_array_type, size) = is_array(type_str)?;
        let is_dynamic_type = is_dynamic(type_str);
        let (is_tuple_type, tuple_types) = is_tuple(type_str)?;

        let encoded_value = handle_offset(encoded_values, cursor, is_dynamic_type, 0);
        let (value, size) = if is_array_type {
            let array_values = decode_array(
                type_str,
                encoded_value,
                size,
                is_dynamic_type,
                is_tuple_type,
                &tuple_types,
            )?;
            let len = array_values.len();
            (Value::Collection(array_values), len)
        } else if is_tuple_type {
            let tuple_values = abi_decode(&tuple_types, &encoded_value.to_vec())?;
            let len = tuple_values.len();
            (Value::Collection(tuple_values), len)
        } else {
            (decode(encoded_value, type_str, is_dynamic_type)?, 1)
        };
        values.push(value);
        cursor += size * 32;
    }

    Ok(values)
}

fn decode_array(
    arr_type_str: &str,
    encoded_values: &[u8],
    size: usize,
    is_dynamic_type: bool,
    is_tuple_type: bool,
    tuple_types: &Vec<&str>,
) -> Result<Vec<Value>, CodecError> {
    let mut encoded_values = encoded_values;
    let mut size = size;
    if size == 0 {
        size = u64::from_be_bytes(encoded_values[24..32].try_into().unwrap()) as usize;
        encoded_values = &encoded_values[32..];
    }
    let type_str = arr_type_str.split("[").next().unwrap();

    let mut values = Vec::new();
    let mut cursor = 0;
    for _ in 0..size {
        if is_tuple_type {
            let tuple_encoded_values =
                handle_offset(encoded_values, cursor, is_dynamic_type, cursor);
            let tuple_values = abi_decode(tuple_types, &tuple_encoded_values.to_vec())?;
            values.push(Value::Collection(tuple_values));
            cursor += 32 * values.len();
        } else {
            let encoded_value = handle_offset(encoded_values, cursor, is_dynamic_type, 0);
            let value = decode(encoded_value, type_str, is_dynamic_type)?;
            values.push(value);
            cursor += 32;
        }
    }

    Ok(values)
}

fn handle_offset(
    encoded_values: &[u8],
    cursor: usize,
    is_dynamic_type: bool,
    tuple_cursor: usize,
) -> &[u8] {
    if is_dynamic_type {
        let offset =
            u64::from_be_bytes(encoded_values[cursor + 24..cursor + 32].try_into().unwrap())
                as usize;
        &encoded_values[offset + tuple_cursor..]
    } else {
        &encoded_values[cursor..cursor + 32]
    }
}

fn decode(
    encoded_value: &[u8],
    type_str: &str,
    is_dynamic_type: bool,
) -> Result<Value, CodecError> {
    let inner_value = if is_dynamic_type {
        let length = u64::from_be_bytes(encoded_value[24..32].try_into().unwrap());
        &encoded_value[32..32 + length as usize]
    } else {
        let length = get_bytes_from_type(type_str);
        &encoded_value[32 - length..32]
    };

    decode_packed(inner_value, type_str)
}

fn decode_packed(encoded_value: &[u8], type_str: &str) -> Result<Value, CodecError> {
    match type_str {
        "address" => Ok(Value::Single(
            Box::new(Address::from_bytes::<20>(
                encoded_value[..20].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes" => Ok(Value::Single(
            Box::new(Bytes::copy_from_slice(encoded_value)),
            type_str.to_string(),
        )),
        "string" => {
            let string = String::from_utf8(encoded_value.to_vec()).unwrap();
            Ok(Value::Single(Box::new(string), type_str.to_string()))
        }
        "bool" => Ok(Value::Single(
            Box::new(bool::from_bytes::<1>(
                encoded_value[..1].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint8" => Ok(Value::Single(
            Box::new(U8::from_bytes::<1>(encoded_value[..1].try_into().unwrap())),
            type_str.to_string(),
        )),
        "uint16" => Ok(Value::Single(
            Box::new(U16::from_bytes::<2>(encoded_value[..2].try_into().unwrap())),
            type_str.to_string(),
        )),
        "uint24" => Ok(Value::Single(
            Box::new(U24::from_bytes::<3>(encoded_value[..3].try_into().unwrap())),
            type_str.to_string(),
        )),
        "uint32" => Ok(Value::Single(
            Box::new(U32::from_bytes::<4>(encoded_value[..4].try_into().unwrap())),
            type_str.to_string(),
        )),
        "uint40" => Ok(Value::Single(
            Box::new(U40::from_bytes::<5>(encoded_value[..5].try_into().unwrap())),
            type_str.to_string(),
        )),
        "uint48" => Ok(Value::Single(
            Box::new(U48::from_bytes::<6>(encoded_value[..6].try_into().unwrap())),
            type_str.to_string(),
        )),
        "uint56" => Ok(Value::Single(
            Box::new(U56::from_bytes::<7>(encoded_value[..7].try_into().unwrap())),
            type_str.to_string(),
        )),
        "uint64" => Ok(Value::Single(
            Box::new(U64::from_bytes::<8>(encoded_value[..8].try_into().unwrap())),
            type_str.to_string(),
        )),
        "uint72" => Ok(Value::Single(
            Box::new(U72::from_bytes::<9>(encoded_value[..9].try_into().unwrap())),
            type_str.to_string(),
        )),
        "uint80" => Ok(Value::Single(
            Box::new(U80::from_bytes::<10>(
                encoded_value[..10].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint88" => Ok(Value::Single(
            Box::new(U88::from_bytes::<11>(
                encoded_value[..11].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint96" => Ok(Value::Single(
            Box::new(U96::from_bytes::<12>(
                encoded_value[..12].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint104" => Ok(Value::Single(
            Box::new(U104::from_bytes::<13>(
                encoded_value[..13].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint112" => Ok(Value::Single(
            Box::new(U112::from_bytes::<14>(
                encoded_value[..14].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint120" => Ok(Value::Single(
            Box::new(U120::from_bytes::<15>(
                encoded_value[..15].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint128" => Ok(Value::Single(
            Box::new(U128::from_bytes::<16>(
                encoded_value[..16].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint136" => Ok(Value::Single(
            Box::new(U136::from_bytes::<17>(
                encoded_value[..17].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint144" => Ok(Value::Single(
            Box::new(U144::from_bytes::<18>(
                encoded_value[..18].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint152" => Ok(Value::Single(
            Box::new(U152::from_bytes::<19>(
                encoded_value[..19].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint160" => Ok(Value::Single(
            Box::new(U160::from_bytes::<20>(
                encoded_value[..20].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint168" => Ok(Value::Single(
            Box::new(U168::from_bytes::<21>(
                encoded_value[..21].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint176" => Ok(Value::Single(
            Box::new(U176::from_bytes::<22>(
                encoded_value[..22].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint184" => Ok(Value::Single(
            Box::new(U184::from_bytes::<23>(
                encoded_value[..23].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint192" => Ok(Value::Single(
            Box::new(U192::from_bytes::<24>(
                encoded_value[..24].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint200" => Ok(Value::Single(
            Box::new(U200::from_bytes::<25>(
                encoded_value[..25].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint208" => Ok(Value::Single(
            Box::new(U208::from_bytes::<26>(
                encoded_value[..26].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint216" => Ok(Value::Single(
            Box::new(U216::from_bytes::<27>(
                encoded_value[..27].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint224" => Ok(Value::Single(
            Box::new(U224::from_bytes::<28>(
                encoded_value[..28].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint232" => Ok(Value::Single(
            Box::new(U232::from_bytes::<29>(
                encoded_value[..29].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint240" => Ok(Value::Single(
            Box::new(U240::from_bytes::<30>(
                encoded_value[..30].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint248" => Ok(Value::Single(
            Box::new(U248::from_bytes::<31>(
                encoded_value[..31].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "uint256" => Ok(Value::Single(
            Box::new(U256::from_bytes::<32>(
                encoded_value[..32].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int8" => Ok(Value::Single(
            Box::new(I8::from_bytes::<1>(encoded_value[..1].try_into().unwrap())),
            type_str.to_string(),
        )),
        "int16" => Ok(Value::Single(
            Box::new(I16::from_bytes::<2>(encoded_value[..2].try_into().unwrap())),
            type_str.to_string(),
        )),
        "int24" => Ok(Value::Single(
            Box::new(I24::from_bytes::<3>(encoded_value[..3].try_into().unwrap())),
            type_str.to_string(),
        )),
        "int32" => Ok(Value::Single(
            Box::new(I32::from_bytes::<4>(encoded_value[..4].try_into().unwrap())),
            type_str.to_string(),
        )),
        "int40" => Ok(Value::Single(
            Box::new(I40::from_bytes::<5>(encoded_value[..5].try_into().unwrap())),
            type_str.to_string(),
        )),
        "int48" => Ok(Value::Single(
            Box::new(I48::from_bytes::<6>(encoded_value[..6].try_into().unwrap())),
            type_str.to_string(),
        )),
        "int56" => Ok(Value::Single(
            Box::new(I56::from_bytes::<7>(encoded_value[..7].try_into().unwrap())),
            type_str.to_string(),
        )),
        "int64" => Ok(Value::Single(
            Box::new(I64::from_bytes::<8>(encoded_value[..8].try_into().unwrap())),
            type_str.to_string(),
        )),
        "int72" => Ok(Value::Single(
            Box::new(I72::from_bytes::<9>(encoded_value[..9].try_into().unwrap())),
            type_str.to_string(),
        )),
        "int80" => Ok(Value::Single(
            Box::new(I80::from_bytes::<10>(
                encoded_value[..10].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int88" => Ok(Value::Single(
            Box::new(I88::from_bytes::<11>(
                encoded_value[..11].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int96" => Ok(Value::Single(
            Box::new(I96::from_bytes::<12>(
                encoded_value[..12].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int104" => Ok(Value::Single(
            Box::new(I104::from_bytes::<13>(
                encoded_value[..13].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int112" => Ok(Value::Single(
            Box::new(I112::from_bytes::<14>(
                encoded_value[..14].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int120" => Ok(Value::Single(
            Box::new(I120::from_bytes::<15>(
                encoded_value[..15].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int128" => Ok(Value::Single(
            Box::new(I128::from_bytes::<16>(
                encoded_value[..16].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int136" => Ok(Value::Single(
            Box::new(I136::from_bytes::<17>(
                encoded_value[..17].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int144" => Ok(Value::Single(
            Box::new(I144::from_bytes::<18>(
                encoded_value[..18].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int152" => Ok(Value::Single(
            Box::new(I152::from_bytes::<19>(
                encoded_value[..19].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int160" => Ok(Value::Single(
            Box::new(I160::from_bytes::<20>(
                encoded_value[..20].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int168" => Ok(Value::Single(
            Box::new(I168::from_bytes::<21>(
                encoded_value[..21].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int176" => Ok(Value::Single(
            Box::new(I176::from_bytes::<22>(
                encoded_value[..22].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int184" => Ok(Value::Single(
            Box::new(I184::from_bytes::<23>(
                encoded_value[..23].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int192" => Ok(Value::Single(
            Box::new(I192::from_bytes::<24>(
                encoded_value[..24].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int200" => Ok(Value::Single(
            Box::new(I200::from_bytes::<25>(
                encoded_value[..25].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int208" => Ok(Value::Single(
            Box::new(I208::from_bytes::<26>(
                encoded_value[..26].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int216" => Ok(Value::Single(
            Box::new(I216::from_bytes::<27>(
                encoded_value[..27].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int224" => Ok(Value::Single(
            Box::new(I224::from_bytes::<28>(
                encoded_value[..28].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int232" => Ok(Value::Single(
            Box::new(I232::from_bytes::<29>(
                encoded_value[..29].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int240" => Ok(Value::Single(
            Box::new(I240::from_bytes::<30>(
                encoded_value[..30].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int248" => Ok(Value::Single(
            Box::new(I248::from_bytes::<31>(
                encoded_value[..31].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "int256" => Ok(Value::Single(
            Box::new(I256::from_bytes::<32>(
                encoded_value[..32].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes1" => Ok(Value::Single(
            Box::new(FixedBytes::<1>::from_bytes::<1>(
                encoded_value[..1].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes2" => Ok(Value::Single(
            Box::new(FixedBytes::<2>::from_bytes::<2>(
                encoded_value[..2].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes3" => Ok(Value::Single(
            Box::new(FixedBytes::<3>::from_bytes::<3>(
                encoded_value[..3].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes4" => Ok(Value::Single(
            Box::new(FixedBytes::<4>::from_bytes::<4>(
                encoded_value[..4].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes5" => Ok(Value::Single(
            Box::new(FixedBytes::<5>::from_bytes::<5>(
                encoded_value[..5].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes6" => Ok(Value::Single(
            Box::new(FixedBytes::<6>::from_bytes::<6>(
                encoded_value[..6].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes7" => Ok(Value::Single(
            Box::new(FixedBytes::<7>::from_bytes::<7>(
                encoded_value[..7].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes8" => Ok(Value::Single(
            Box::new(FixedBytes::<8>::from_bytes::<8>(
                encoded_value[..8].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes9" => Ok(Value::Single(
            Box::new(FixedBytes::<9>::from_bytes::<9>(
                encoded_value[..9].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes10" => Ok(Value::Single(
            Box::new(FixedBytes::<10>::from_bytes::<10>(
                encoded_value[..10].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes11" => Ok(Value::Single(
            Box::new(FixedBytes::<11>::from_bytes::<11>(
                encoded_value[..11].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes12" => Ok(Value::Single(
            Box::new(FixedBytes::<12>::from_bytes::<12>(
                encoded_value[..12].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes13" => Ok(Value::Single(
            Box::new(FixedBytes::<13>::from_bytes::<13>(
                encoded_value[..13].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes14" => Ok(Value::Single(
            Box::new(FixedBytes::<14>::from_bytes::<14>(
                encoded_value[..14].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes15" => Ok(Value::Single(
            Box::new(FixedBytes::<15>::from_bytes::<15>(
                encoded_value[..15].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes16" => Ok(Value::Single(
            Box::new(FixedBytes::<16>::from_bytes::<16>(
                encoded_value[..16].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes17" => Ok(Value::Single(
            Box::new(FixedBytes::<17>::from_bytes::<17>(
                encoded_value[..17].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes18" => Ok(Value::Single(
            Box::new(FixedBytes::<18>::from_bytes::<18>(
                encoded_value[..18].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes19" => Ok(Value::Single(
            Box::new(FixedBytes::<19>::from_bytes::<19>(
                encoded_value[..19].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes20" => Ok(Value::Single(
            Box::new(FixedBytes::<20>::from_bytes::<20>(
                encoded_value[..20].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes21" => Ok(Value::Single(
            Box::new(FixedBytes::<21>::from_bytes::<21>(
                encoded_value[..21].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes22" => Ok(Value::Single(
            Box::new(FixedBytes::<22>::from_bytes::<22>(
                encoded_value[..22].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes23" => Ok(Value::Single(
            Box::new(FixedBytes::<23>::from_bytes::<23>(
                encoded_value[..23].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes24" => Ok(Value::Single(
            Box::new(FixedBytes::<24>::from_bytes::<24>(
                encoded_value[..24].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes25" => Ok(Value::Single(
            Box::new(FixedBytes::<25>::from_bytes::<25>(
                encoded_value[..25].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes26" => Ok(Value::Single(
            Box::new(FixedBytes::<26>::from_bytes::<26>(
                encoded_value[..26].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes27" => Ok(Value::Single(
            Box::new(FixedBytes::<27>::from_bytes::<27>(
                encoded_value[..27].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes28" => Ok(Value::Single(
            Box::new(FixedBytes::<28>::from_bytes::<28>(
                encoded_value[..28].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes29" => Ok(Value::Single(
            Box::new(FixedBytes::<29>::from_bytes::<29>(
                encoded_value[..29].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes30" => Ok(Value::Single(
            Box::new(FixedBytes::<30>::from_bytes::<30>(
                encoded_value[..30].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes31" => Ok(Value::Single(
            Box::new(FixedBytes::<31>::from_bytes::<31>(
                encoded_value[..31].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        "bytes32" => Ok(Value::Single(
            Box::new(FixedBytes::<32>::from_bytes::<32>(
                encoded_value[..32].try_into().unwrap(),
            )),
            type_str.to_string(),
        )),
        _ => Err(CodecError::UnsupportedType(type_str.to_string())),
    }
}

#[cfg(test)]
mod encode_tests {
    use super::*;
    use alloy_primitives::hex;

    #[test]
    fn test_decode() {
        let mut value = hex!(
            "0x000000000000000000000000000000000000000000000000000000000000000c48656c6c6f20576f726c64210000000000000000000000000000000000000000"
        );
        let value = decode(&mut value[..], "string", true).unwrap();
        println!("{:?}", value);
        assert!(false);
    }

    #[test]
    fn test_decode_packed() {
        let mut value = hex!("0x000000000000000000000000000000000000000000000000000000000000000c");
        let value = decode_packed(&mut value[..], "uint256").unwrap();
        println!("{:?}", value);
        assert!(false);
    }

    #[test]
    fn test_abi_decode() {
        let value = hex!(
            "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000"
        );
        let type_strs = vec!["address", "(string[],uint256,uint8)[]", "uint256"];
        let value = abi_decode(&type_strs, &value.to_vec()).unwrap();
        println!("{:?}", value);
        assert!(false);
    }
}

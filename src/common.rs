use crate::codec::traits::EncodeCodec;
use crate::errors::CodecError;

pub fn is_dynamic(t: &str) -> bool {
    t.contains("[]") || t.contains("bytes") || t.contains("string")
}

pub fn is_array(t: &str) -> Result<(bool, usize), CodecError> {
    let count_open_brackets = t.chars().filter(|c| *c == '[').count();
    if count_open_brackets != t.chars().filter(|c| *c == ']').count() {
        return Err(CodecError::InvalidArray(t.to_string()));
    }

    if count_open_brackets > 0 {
        let open_brackets_index = t.rfind('[').map_or(-1, |i| i as isize);
        let close_brackets_index = t.rfind(']').map_or(-1, |i| i as isize);
        let close_parenthesis_index = t.rfind(')').map_or(-1, |i| i as isize);

        if open_brackets_index < close_parenthesis_index
            || close_brackets_index < close_parenthesis_index
        {
            return Ok((false, 0));
        }

        let mut array_size: usize = 0;
        if close_brackets_index > open_brackets_index + 1 {
            array_size = match t[(open_brackets_index + 1) as usize..close_brackets_index as usize]
                .parse()
            {
                Ok(size) => size,
                Err(_) => return Err(CodecError::InvalidArray(t.to_string())),
            };
        }

        return Ok((true, array_size));
    }

    Ok((false, 0))
}

pub fn is_tuple(t: &str) -> Result<(bool, Vec<&str>), CodecError> {
    let count_open_parenthesis = t.chars().filter(|c| *c == '(').count();

    if count_open_parenthesis != t.chars().filter(|c| *c == ')').count() {
        return Err(CodecError::InvalidTuple(t.to_string()));
    }

    if count_open_parenthesis > 0 {
        let parameter_types =
            split_parameter_types(&t[t.find('(').unwrap() + 1..t.rfind(')').unwrap()]);
        return Ok((true, parameter_types));
    }

    Ok((false, vec![]))
}

pub fn get_parameter_types(t: &str) -> Result<Vec<&str>, CodecError> {
    if t.chars().filter(|c| *c == '(').count() != t.chars().filter(|c| *c == ')').count() {
        return Err(CodecError::InvalidFunctionSignature(t.to_string()));
    }

    let parameter_types =
        split_parameter_types(&t[t.find('(').unwrap() + 1..t.rfind(')').unwrap()]);

    Ok(parameter_types)
}

pub fn split_parameter_types(t: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    let chars: Vec<char> = t.chars().collect();

    // Remove the outer parentheses if they exist
    let (start_idx, end_idx) = if t.starts_with('(') && t.ends_with(')') {
        (1, t.len() - 1)
    } else {
        (0, t.len())
    };

    for (i, &c) in chars[start_idx..end_idx].iter().enumerate() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                let part = t[start + start_idx..start_idx + i].trim();
                if !part.is_empty() {
                    result.push(part);
                }
                start = i + 1;
            }
            _ => {}
        }
    }

    // Add the last part
    let last_part = t[start + start_idx..end_idx].trim();
    if !last_part.is_empty() {
        result.push(last_part);
    }

    result
}

pub fn check_type_and_value<T: EncodeCodec>(t: &str, v: &T) -> bool {
    t == v.eth_type()
        && match t {
            "uint8" | "int8" | "bool" | "bytes1" => v.bytes_length() == 1,
            "uint16" | "int16" | "bytes2" => v.bytes_length() == 2,
            "uint24" | "int24" | "bytes3" => v.bytes_length() == 3,
            "uint32" | "int32" | "bytes4" => v.bytes_length() == 4,
            "uint40" | "int40" | "bytes5" => v.bytes_length() == 5,
            "uint48" | "int48" | "bytes6" => v.bytes_length() == 6,
            "uint56" | "int56" | "bytes7" => v.bytes_length() == 7,
            "uint64" | "int64" | "bytes8" => v.bytes_length() == 8,
            "uint72" | "int72" | "bytes9" => v.bytes_length() == 9,
            "uint80" | "int80" | "bytes10" => v.bytes_length() == 10,
            "uint88" | "int88" | "bytes11" => v.bytes_length() == 11,
            "uint96" | "int96" | "bytes12" => v.bytes_length() == 12,
            "uint104" | "int104" | "bytes13" => v.bytes_length() == 13,
            "uint112" | "int112" | "bytes14" => v.bytes_length() == 14,
            "uint120" | "int120" | "bytes15" => v.bytes_length() == 15,
            "uint128" | "int128" | "bytes16" => v.bytes_length() == 16,
            "uint136" | "int136" | "bytes17" => v.bytes_length() == 17,
            "uint144" | "int144" | "bytes18" => v.bytes_length() == 18,
            "uint152" | "int152" | "bytes19" => v.bytes_length() == 19,
            "uint160" | "int160" | "bytes20" | "address" => v.bytes_length() == 20,
            "uint168" | "int168" | "bytes21" => v.bytes_length() == 21,
            "uint176" | "int176" | "bytes22" => v.bytes_length() == 22,
            "uint184" | "int184" | "bytes23" => v.bytes_length() == 23,
            "uint192" | "int192" | "bytes24" => v.bytes_length() == 24,
            "uint200" | "int200" | "bytes25" => v.bytes_length() == 25,
            "uint208" | "int208" | "bytes26" => v.bytes_length() == 26,
            "uint216" | "int216" | "bytes27" => v.bytes_length() == 27,
            "uint224" | "int224" | "bytes28" => v.bytes_length() == 28,
            "uint232" | "int232" | "bytes29" => v.bytes_length() == 29,
            "uint240" | "int240" | "bytes30" => v.bytes_length() == 30,
            "uint248" | "int248" | "bytes31" => v.bytes_length() == 31,
            "uint256" | "int256" | "bytes32" => v.bytes_length() == 32,
            "string" | "bytes" => true,
            _ => false,
        }
}

#[cfg(test)]
mod common_tests {
    use super::*;

    #[test]
    fn split_parameter_types_1() {
        let signatre = "(uint256,address,(uint256[],bytes)[],address,uint8)";
        let result = split_parameter_types(signatre);
        assert_eq!(
            result,
            vec![
                "uint256",
                "address",
                "(uint256[],bytes)[]",
                "address",
                "uint8"
            ]
        );
    }

    #[test]
    fn split_parameter_types_2() {
        let signatre = "(uint256,(address,(uint256[],bytes)[],address)[],uint8,string[])";
        let result = split_parameter_types(signatre);
        assert_eq!(
            result,
            vec![
                "uint256",
                "(address,(uint256[],bytes)[],address)[]",
                "uint8",
                "string[]"
            ]
        );
    }

    #[test]
    fn get_parameter_types_success_1() {
        let signature = "blabla(uint256,address,(uint256[],bytes)[],address,uint8)";
        let result = get_parameter_types(signature).unwrap();
        assert_eq!(
            result,
            vec![
                "uint256",
                "address",
                "(uint256[],bytes)[]",
                "address",
                "uint8"
            ]
        );
    }

    #[test]
    fn get_parameter_types_success_2() {
        let signature = "blabla(uint64,address,((uint256,address)[],(uint256[],bytes)[],address)[],uint8,string[])";
        let result = get_parameter_types(signature).unwrap();
        assert_eq!(
            result,
            vec![
                "uint64",
                "address",
                "((uint256,address)[],(uint256[],bytes)[],address)[]",
                "uint8",
                "string[]"
            ]
        );
    }

    #[test]
    fn get_parameter_types_error_1() {
        let signature = "blablauint64,address)";
        let result = get_parameter_types(signature).expect_err("Invalid function signature");
        assert_eq!(
            result,
            CodecError::InvalidFunctionSignature(signature.to_string())
        );
    }

    #[test]
    fn get_parameter_types_error_2() {
        let signature = "blabla(uint64,address";
        let result = get_parameter_types(signature).expect_err("Invalid function signature");
        assert_eq!(
            result,
            CodecError::InvalidFunctionSignature(signature.to_string())
        );
    }

    #[test]
    fn get_parameter_types_error_3() {
        let signature = "blabla(uint64,address,((uint256,address)[],(uint256[],bytes)[],address)[],uint8,string[]";
        let result = get_parameter_types(signature).expect_err("Invalid function signature");
        assert_eq!(
            result,
            CodecError::InvalidFunctionSignature(signature.to_string())
        );
    }

    #[test]
    fn is_dynamic_1() {
        let result = is_dynamic("address,uint256[]");
        assert_eq!(result, true);
    }

    #[test]
    fn is_dynamic_2() {
        let result = is_dynamic("uint256,bytes");
        assert_eq!(result, true);
    }

    #[test]
    fn is_dynamic_3() {
        let result = is_dynamic("address,string");
        assert_eq!(result, true);
    }

    #[test]
    fn is_dynamic_4() {
        let result = is_dynamic("address[3]");
        assert_eq!(result, false);
    }

    #[test]
    fn is_dynamic_5() {
        let result = is_dynamic("address,bytes[3],uint256");
        assert_eq!(result, true);
    }

    #[test]
    fn is_array_success_1() {
        let result = is_array("address[3]");
        assert_eq!(result, Ok((true, 3)));
    }

    #[test]
    fn is_array_success_2() {
        let result = is_array("address[]");
        assert_eq!(result, Ok((true, 0)));
    }

    #[test]
    fn is_array_success_3() {
        let result = is_array("address");
        assert_eq!(result, Ok((false, 0)));
    }

    #[test]
    fn is_array_error_1() {
        let result = is_array("address[");
        assert_eq!(
            result,
            Err(CodecError::InvalidArray("address[".to_string()))
        );
    }

    #[test]
    fn is_tuple_success_1() {
        let result = is_tuple("(uint256,address,(uint256[],bytes)[],address,uint8)");
        assert_eq!(
            result,
            Ok((
                true,
                vec![
                    "uint256",
                    "address",
                    "(uint256[],bytes)[]",
                    "address",
                    "uint8"
                ]
            ))
        );
    }

    #[test]
    fn is_tuple_error_1() {
        let result = is_tuple("(uint256,address,(uint256[],bytes)[],address,uint8");
        assert_eq!(
            result,
            Err(CodecError::InvalidTuple(
                "(uint256,address,(uint256[],bytes)[],address,uint8".to_string()
            ))
        );
    }
}

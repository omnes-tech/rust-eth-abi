use crate::codec::types::Value;

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

pub fn get_collection_i(values: &Vec<Value>, index: usize) -> Vec<Value> {
    match &values[index] {
        Value::Single(_, _) => vec![values[index].clone()],
        Value::Collection(vals) => vals.to_vec(),
    }
}

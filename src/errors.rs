#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CodecError {
    // common
    #[error("Invalid array: {0}")]
    InvalidArray(String),
    #[error("Invalid tuple: {0}")]
    InvalidTuple(String),
    #[error("Invalid function signature: {0}")]
    InvalidFunctionSignature(String),

    // encode
    #[error("Invalid type and value: {0}")]
    InvalidTypeAndValue(String, String),
    #[error("Lengths mismatch: {0} != {1}")]
    LengthsMismatch(usize, usize),

    // decode
    #[error("Invalid value length: {0}")]
    InvalidValueLength(usize),
    #[error("Unsupported type: {0}")]
    UnsupportedType(String),
    #[error("Invalid selector")]
    InvalidSelector,
}

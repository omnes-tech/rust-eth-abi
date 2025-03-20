use crate::codec::traits::{DecodeCodec, EncodeCodec};
use alloy_primitives::aliases::*;
use std::any::Any;

macro_rules! impl_encode_codec_for_uint_and_int {
    ($($t:ty),*) => {
        $(
            impl EncodeCodec for $t {
                fn to_bytes_vec(&self) -> Vec<u8> {
                    self.to_be_bytes::<{<$t>::BYTES}>().to_vec()
                }

                fn bytes_length(&self) -> usize {
                    Self::BYTES
                }

                fn eth_type(&self) -> String {
                    let type_name = std::any::type_name::<Self>();
                    let prefix = if type_name.contains("U") { "uint" } else { "int" };
                    format!("{}{}", prefix, Self::BYTES * 8)
                }

                fn to_string(&self) -> String {
                    format!("{}", self)
                }

                fn as_any(&self) -> &dyn Any {
                    self
                }
            }
        )*
    };
}

macro_rules! impl_decode_codec_for_uint_and_int {
    ($($t:ty),*) => {
        $(
            impl DecodeCodec for $t {
                fn from_bytes<const BYTES: usize>(bytes: [u8; BYTES]) -> Self {
                    Self::from_be_bytes(bytes)
                }
            }
        )*
    };
}

impl_encode_codec_for_uint_and_int!(
    U8, U16, U24, U32, U40, U48, U56, U64, U72, U80, U88, U96, U104, U112, U120, U128, U136, U144,
    U152, U160, U168, U176, U184, U192, U200, U208, U216, U224, U232, U240, U248, U256, I8, I16,
    I24, I32, I40, I48, I56, I64, I72, I80, I88, I96, I104, I112, I120, I128, I136, I144, I152,
    I160, I168, I176, I184, I192, I200, I208, I216, I224, I232, I240, I248, I256
);

impl_decode_codec_for_uint_and_int!(
    U8, U16, U24, U32, U40, U48, U56, U64, U72, U80, U88, U96, U104, U112, U120, U128, U136, U144,
    U152, U160, U168, U176, U184, U192, U200, U208, U216, U224, U232, U240, U248, U256, I8, I16,
    I24, I32, I40, I48, I56, I64, I72, I80, I88, I96, I104, I112, I120, I128, I136, I144, I152,
    I160, I168, I176, I184, I192, I200, I208, I216, I224, I232, I240, I248, I256
);

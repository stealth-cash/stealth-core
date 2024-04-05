use anchor_lang::prelude::*;

pub fn anchor_err(error_msg: &str) -> AnchorError {
    AnchorError {
        error_msg: error_msg.to_string(),
        error_name: "AnchorError".to_string(),
        error_code_number: 0,
        error_origin: None,
        compared_values: None,
    }
}

pub fn pubkey_to_u128(pubkey: &Pubkey) -> u128 {
    let bytes = pubkey.to_bytes();

    let mut result: u128 = 0;
    for &byte in &bytes[..16] {
        result = (result << 8) | byte as u128;
    }
    result
}
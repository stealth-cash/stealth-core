use anchor_lang::solana_program::pubkey::Pubkey;

pub fn vec_to_u128(vec: &Vec<u8>) -> u128 {
    assert_eq!(vec.len(), 16, "Vector length must be 16 bytes");
    let mut array = [0u8; 16];
    array.copy_from_slice(&vec);
    u128::from_be_bytes(array)
}

pub fn pubkey_to_u128(pubkey: &Pubkey) -> u128 {
    let bytes = pubkey.to_bytes();

    let mut result: u128 = 0;
    for &byte in &bytes[..16] {
        result = (result << 8) | byte as u128;
    }
    result
}

use std::collections::HashMap;
use std::str::FromStr;
use anchor_lang::prelude::*;

use crate::{hasher::Hasher, uint256::Uint256};
use crate::utils;

pub const ROOT_HISTORY_SIZE: u8 = 30;

#[derive(Debug, Clone)]
pub struct MerkleTree {
    levels: u8,
    filled_subtrees: HashMap<u8, Uint256>,
    roots: HashMap<u8, Uint256>,
    current_root_index: u8,
    next_index: u8
}

impl ToString for MerkleTree {
    fn to_string(&self) -> String {
        let mut string_representation = String::new();
        
        string_representation.push_str(&format!("levels: {}\n", self.levels));
        
        string_representation.push_str("filled_subtrees:\n");
        for (level, value) in &self.filled_subtrees {
            string_representation.push_str(&format!("  {}: {}\n", level, value.to_string()));
        }
        
        string_representation.push_str("roots:\n");
        for (level, value) in &self.roots {
            string_representation.push_str(&format!("  {}: {}\n", level, value.to_string()));
        }
        
        string_representation.push_str(&format!("current_root_index: {}\n", self.current_root_index));
        string_representation.push_str(&format!("next_index: {}\n", self.next_index));
        
        string_representation
    }
}

impl FromStr for MerkleTree {
    type Err = AnchorError;

    fn from_str(s: &str) -> std::result::Result<Self, AnchorError> {
        let mut levels: Option<u8> = None;
        let mut filled_subtrees: HashMap<u8, Uint256> = HashMap::new();
        let mut roots: HashMap<u8, Uint256> = HashMap::new();
        let mut current_root_index: Option<u8> = None;
        let mut next_index: Option<u8> = None;

        for line in s.lines() {
            let parts: Vec<&str> = line.trim().splitn(2, ":").collect();
            if parts.len() != 2 {
                return Err(utils::err("Error").into()
                );
            }
            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "levels" => {
                    levels = Some(value.parse().map_err(|e| format!("Parsing levels failed: {}", e)).unwrap());
                }
                "filled_subtrees" => {
                    let level_value: Vec<&str> = value.splitn(2, ":").collect();
                    if level_value.len() != 2 {
                        return Err(utils::err("Error occured in filled subtrees").into());
                    }
                    let level: u8 = level_value[0].trim().parse().map_err(|e| format!("Parsing filled_subtrees level failed: {}", e)).unwrap();
                    let value: Uint256 = level_value[1].trim().parse().map_err(|e| format!("Parsing filled_subtrees value failed: {}", e)).unwrap();
                    filled_subtrees.insert(level, value);
                }
                "roots" => {
                    let level_value: Vec<&str> = value.splitn(2, ":").collect();
                    if level_value.len() != 2 {
                        return Err(utils::err("Error in roots").into());
                    }
                    let level: u8 = level_value[0].trim().parse().map_err(|e| format!("Parsing roots level failed: {}", e)).unwrap();
                    let value: Uint256 = level_value[1].trim().parse().map_err(|e| format!("Parsing roots value failed: {}", e)).unwrap();
                    roots.insert(level, value);
                }
                "current_root_index" => {
                    current_root_index = Some(value.parse().map_err(|e| format!("Parsing current_root_index failed: {}", e)).unwrap());
                }
                "next_index" => {
                    next_index = Some(value.parse().map_err(|e| format!("Parsing next_index failed: {}", e)).unwrap());
                }
                _ => {
                    return Err(utils::err("Unexpecte error").into());
                }
            }
        }

        let levels = levels.ok_or("Missing levels").unwrap();
        let current_root_index = current_root_index.ok_or("Missing current_root_index").unwrap();
        let next_index = next_index.ok_or("Missing next_index").unwrap();

        Ok(MerkleTree {
            levels,
            filled_subtrees,
            roots,
            current_root_index,
            next_index
        })
    }
}

impl MerkleTree {
    pub fn new(levels: u8) -> Self {
        let mut instance = MerkleTree {
            levels,
            filled_subtrees: HashMap::new(),
            roots: HashMap::new(),
            current_root_index: 0,
            next_index: 0
        };

        for i in 0..levels {
            instance.filled_subtrees.insert(i, zeros(i));
        }

        instance.roots.insert(0, zeros(levels - 1));
        instance
    }

    pub fn hash_left_right(&self, left: Uint256, right: Uint256) -> Uint256 {
        let field_size: Uint256 = Uint256::from("21888242871839275222246405745257275088548364400416034343698204186575808495617");

        let mut r = left;
        let c = Uint256::new(0);

        r = Hasher::mimc_sponge(&r, &c, &field_size);        
        r = r.add_mod(&right, &field_size);
        r = Hasher::mimc_sponge(&r, &c, &field_size);

        r
    }

    pub fn insert(&mut self, leaf: Uint256) -> Result<u8> {
        if self.next_index < 2_u8.pow(self.levels as u32) {
            return Err(utils::err("Merkle tree is full, no more leaves can be added").into());
        }

        let _next_index = self.next_index;
        let mut current_index = self.next_index;
        let mut current_level_hash = leaf.clone();
        let mut left: Uint256;
        let mut right: Uint256;

        for i in 0..self.levels {
            if current_index % 2 == 0 {
                left = current_level_hash.clone();
                right = zeros(i);
                self.filled_subtrees.insert(i, current_level_hash.clone());
            } else {
                left = self.filled_subtrees.get(&i).unwrap().clone();
                right = current_level_hash.clone();
            }
            current_level_hash = self.hash_left_right(left, right);
            current_index /= 2;
        }

        let new_root_index = (self.current_root_index + 1) % ROOT_HISTORY_SIZE;
        self.current_root_index = new_root_index;
        self.roots.insert(new_root_index, current_level_hash.clone());
        self.next_index = _next_index + 1;

        Ok(_next_index)
    }

    pub fn is_known_root(&self, root: Uint256) -> bool {
        if root.is_zero() {
            return false;
        }
    
        let current_root_index = self.current_root_index;
        let mut i = current_root_index;
        
        loop {
            if self.roots.get(&i).is_some() && *self.roots.get(&i).unwrap() == root {
                return true;
            }
            if i == 0 {
                i = ROOT_HISTORY_SIZE - 1;
            } else {
                i -= 1;
            }
            if i == current_root_index {
                break;
            }
        }
        false
    }

    pub fn get_last_root(&self) -> Uint256 {
        return self.roots.get(&self.current_root_index).unwrap().clone();
    }
}

pub fn zeros(i: u8) -> Uint256 {
    match i {
        0 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x2f, 0xe5, 0x4c, 0x60, 0xd3, 0xac, 0xab, 0xf3, 
            0x34, 0x3a, 0x35, 0xb6, 0xeb, 0xa1, 0x5d, 0xb4, 
            0x82, 0x1b, 0x34, 0x0f, 0x76, 0xe7, 0x41, 0xe2, 
            0x24, 0x96, 0x85, 0xed, 0x48, 0x99, 0xaf, 0x6c
        ])),
        1 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x25, 0x6a, 0x61, 0x35, 0x77, 0x7e, 0xe2, 0xfd, 
            0x26, 0xf5, 0x4b, 0x8b, 0x70, 0x37, 0xa2, 0x54, 
            0x39, 0xd5, 0x23, 0x5c, 0xae, 0xe2, 0x24, 0x15, 
            0x41, 0x86, 0xd2, 0xb8, 0xa5, 0x2e, 0x31, 0x0d
        ])),
        2 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x11, 0x51, 0x94, 0x98, 0x95, 0xe8, 0x2a, 0xb1, 
            0x99, 0x24, 0xde, 0x92, 0xc4, 0xa3, 0xd6, 0xf7, 
            0xbc, 0xb6, 0xd, 0x92, 0xb0, 0x05, 0x04, 0xb8, 
            0x19, 0x96, 0x13, 0x68, 0x3f, 0xc, 0x2, 0x0
        ])),
        3 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x20, 0x12, 0x1e, 0xe8, 0x11, 0x48, 0x9f, 0xf8,
            0xd6, 0x1f, 0x09, 0xfb, 0x89, 0xe3, 0x13, 0xf1,
            0x49, 0x59, 0xa0, 0xf2, 0x8b, 0xb4, 0x28, 0xa2,
            0x0d, 0xba, 0x6b, 0xb0, 0xb0, 0x68, 0xb3, 0xbd
        ])),
        4 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x0a, 0x89, 0xca, 0x6f, 0xfa, 0x14, 0xcc, 0x46,
            0x2c, 0xfe, 0xdb, 0x84, 0x2c, 0x30, 0xed, 0x22,
            0x1a, 0x50, 0xa3, 0xd6, 0xbf, 0x02, 0x2a, 0x6a,
            0x57, 0xdc, 0x82, 0xab, 0x24, 0xc1, 0x57, 0xc9
        ])),
        5 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x24, 0xca, 0x05, 0xc2, 0xb5, 0xcd, 0x42, 0xe8,
            0x90, 0xd6, 0xbe, 0x94, 0xc6, 0x8d, 0x06, 0x89,
            0xf4, 0xf2, 0x1c, 0x9c, 0xec, 0x9c, 0x0f, 0x13,
            0xfe, 0x41, 0xd5, 0x66, 0xdf, 0xb5, 0x49, 0x59
        ])),
        6 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x1c, 0xcb, 0x97, 0xc9, 0x32, 0x56, 0x5a, 0x92,
            0xc6, 0x01, 0x56, 0xbd, 0xba, 0x2d, 0x08, 0xf3,
            0xbf, 0x13, 0x77, 0x46, 0x4e, 0x02, 0x5c, 0xee,
            0x76, 0x56, 0x79, 0xe6, 0x04, 0xa7, 0x31, 0x5c
        ])),
        7 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x19, 0x15, 0x6f, 0xbd, 0x7d, 0x1a, 0x8b, 0xf5,
            0xcb, 0xa8, 0x90, 0x93, 0x67, 0xde, 0x1b, 0x62,
            0x45, 0x34, 0xeb, 0xab, 0x4f, 0xf, 0x79, 0xe0,
            0x3, 0xbc, 0xcd, 0xd1, 0xb1, 0x82, 0xbd, 0xb4        
        ])),
        8 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x26, 0x1a, 0xf8, 0xc1, 0xf0, 0x91, 0x2e, 0x46,
            0x57, 0x44, 0x64, 0x14, 0x9f, 0x62, 0x2d, 0x46,
            0x6c, 0x39, 0x20, 0xac, 0x6e, 0x5f, 0xf3, 0x7e,
            0x36, 0x60, 0x4c, 0xb1, 0x1d, 0xff, 0xf8, 0x0
        ])),
        9 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x0, 0x58, 0x45, 0x97, 0x24, 0xff, 0x6c, 0xa5,
            0xa1, 0x65, 0x2f, 0xcb, 0xbc, 0x3e, 0x82, 0xb9,
            0x38, 0x95, 0xcf, 0x8, 0xe9, 0x75, 0xb1, 0x9b,
            0xea, 0xb3, 0xf5, 0x4c, 0x21, 0x7d, 0x1c, 0x0
        ])),
        10 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x1f, 0x04, 0xef, 0x20,
            0xde, 0xe4, 0x8d, 0x39,
            0x98, 0x4d, 0x8e, 0xab,
            0xe7, 0x68, 0xa7, 0x0e,
            0xaf, 0xa6, 0x31, 0x0a,
            0xd2, 0xad, 0x1e, 0x30,
            0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00
        ])),
        11 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x1b, 0xea, 0x3d, 0xec, 0x5d, 0xab, 0x51, 0x56,
            0x7c, 0xe7, 0xe2, 0x0, 0xa3, 0xa, 0x30, 0xf7,
            0xba, 0x6d, 0x42, 0x76, 0xae, 0xaa, 0x53, 0xe2,
            0x68, 0x6f, 0x96, 0x2a, 0x46, 0xc6, 0x6d, 0x51
        ])),
        12 => Uint256::new(utils::vec_to_u128(
            &vec![
            0xe, 0xe0, 0xf9, 0x41, 0xe2, 0xda, 0x4b, 0x9e,
            0x31, 0xc3, 0xca, 0x97, 0xa4, 0xd, 0x8f, 0xa9,
            0xce, 0x68, 0xd9, 0x7c, 0x8, 0x41, 0x77, 0x7,
            0x1b, 0x3c, 0xb4, 0x6c, 0xd3, 0x37, 0x2f, 0xf
        ])),
        13 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x1c, 0xa9, 0x50, 0x3e,
            0x89, 0x35, 0x88, 0x45,
            0x1b, 0xba, 0xf2, 0x0e,
            0x14, 0xeb, 0x4c, 0x46,
            0xb8, 0x9c, 0x46, 0xb8,
            0x97, 0x72, 0x97, 0xb9,
            0x6e, 0x3b, 0x2e, 0xbf,
            0x3a, 0x36, 0xa9, 0x48,
        ])),
        14 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x13, 0x3a, 0x80, 0xe3, 0x6, 0x97, 0xcd, 0x55, 
            0xd8, 0xf7, 0xd4, 0xb0, 0x96, 0x5b, 0x7b, 0xe2, 
            0x40, 0x57, 0xba, 0x5d, 0xc3, 0xda, 0x89, 0x8e, 
            0xe2, 0x18, 0x72, 0x32, 0x44, 0x6c, 0xb1, 0x8
        ])),
        15 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x13, 0xe6, 0xd8, 0xfc, 0x88, 0x83, 0x9e, 0xd7, 
            0x6e, 0x18, 0x2c, 0x2a, 0x77, 0x9a, 0xf5, 0xb2, 
            0xc0, 0xda, 0x9d, 0xd1, 0x8, 0xc9, 0x4, 0x27, 
            0xa6, 0x44, 0xf7, 0xe1, 0x48, 0xa6, 0x25, 0x3b
        ])),
        16 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x1e, 0xb1, 0x6b, 0x5, 0x7a, 0x47, 0x7f, 0x4b, 
            0xc8, 0xf5, 0x72, 0xea, 0x6b, 0xee, 0x39, 0x56, 
            0x10, 0x98, 0xf7, 0x8f, 0x15, 0xbf, 0xb3, 0x69, 
            0x9d, 0xcb, 0xb7, 0xbd, 0x8d, 0xb6, 0x18, 0x54
        ])),
        17 => Uint256::new(utils::vec_to_u128(
            &vec![
            0xd, 0xa2, 0xcb, 0x16, 0xa1, 0xce, 0xaa, 0xbf, 
            0x1c, 0x16, 0xb8, 0x38, 0xf7, 0xa9, 0xe3, 0xf2, 
            0xa3, 0xa3, 0x8, 0x8d, 0x9e, 0xa, 0x6d, 0xeb, 
            0xaa, 0x74, 0x81, 0x14, 0x62, 0x6, 0x96, 0xea
        ])),
        18 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x24, 0xa3, 0xb3, 0xd8, 0x22, 0x42, 0xb, 0x14, 
            0xb5, 0xd8, 0xcb, 0x6c, 0x28, 0xa5, 0x74, 0xf0, 
            0x1e, 0x98, 0xea, 0x9e, 0x94, 0x5, 0x51, 0xd2, 
            0xeb, 0xd7, 0x5c, 0xee, 0x12, 0x64, 0x9f, 0x9d
        ])),
        19 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x19, 0x86, 0x22, 0xac, 0xbd, 0x78, 0x3d, 0x1b, 
            0xd, 0x90, 0x64, 0x10, 0x5b, 0x1f, 0xc8, 0xe4, 
            0xd8, 0x88, 0x9d, 0xe9, 0x5c, 0x4c, 0x51, 0x9b, 
            0x3f, 0x63, 0x58, 0x9f, 0xe6, 0xaf, 0xc0, 0x5
        ])),
        20 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x29, 0xd7, 0xed, 0x39, 0x12, 0x56, 0xcc, 0xc3, 
            0xea, 0x59, 0x6c, 0x86, 0xe9, 0x33, 0xb8, 0x9f, 
            0xf3, 0x39, 0xd2, 0x5e, 0xa8, 0xdd, 0xce, 0xd9, 
            0x75, 0xae, 0x2f, 0xe3, 0xb, 0x52, 0x96, 0xd4
        ])),
        21 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x19, 0xbe, 0x59, 0xf2, 0xf0, 0x41, 0x3c, 0xe7, 
            0x8c, 0xc0, 0xc3, 0x70, 0x3a, 0x3a, 0x54, 0x51, 
            0xb1, 0xd7, 0xf3, 0x96, 0x29, 0xfa, 0x33, 0xab, 
            0xd1, 0x15, 0x48, 0xa7, 0x60, 0x65, 0xb2, 0x96
        ])),
        22 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x1f, 0xf3, 0xf6, 0x17, 0x97, 0xe5, 0x38, 0xb7, 
            0xe, 0x61, 0x93, 0x10, 0xd3, 0x3f, 0x2a, 0x6, 
            0x3e, 0x7e, 0xb5, 0x91, 0x4, 0xe1, 0x12, 0xe9, 
            0x57, 0x38, 0xda, 0x12, 0x54, 0xdc, 0x34, 0x53
        ])),
        23 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x10, 0xc1, 0x6a, 0xe9, 0x95, 0x9c, 0xf8, 0x35, 
            0x89, 0x80, 0xd9, 0xdd, 0x96, 0x16, 0xe4, 0x82, 
            0x28, 0x73, 0x73, 0x10, 0xa1, 0xe, 0x2b, 0x6b, 
            0x73, 0x1c, 0x1a, 0x54, 0x8f, 0x3, 0x6c, 0x48
        ])),
        24 => Uint256::new(utils::vec_to_u128(
            &vec![
            0xb, 0xa4, 0x33, 0xa6, 0x31, 0x74, 0xa9, 0xa, 
            0xc2, 0x9, 0x92, 0xe7, 0x5e, 0x30, 0x95, 0x49, 
            0x68, 0x12, 0xb6, 0x52, 0x68, 0x5b, 0x5e, 0x1a, 
            0x2e, 0xae, 0xb, 0x1b, 0xf4, 0xe8, 0xfc, 0xd1
        ])),
        25 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x01, 0x9d, 0xdb, 0x9d,
            0xf2, 0xbc, 0x98, 0xd9,
            0x87, 0xd0, 0xdf, 0xec,
            0xa9, 0xd2, 0xb6, 0x43,
            0xde, 0xaf, 0xab, 0x2b,
            0x64, 0x3d, 0xea, 0xfa,
            0xb8, 0xf7, 0x03, 0x65,
            0x62, 0xe6, 0x27, 0xc3,
        ])),
        26 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x2d, 0x3c, 0x88, 0xb2, 
            0x31, 0x75, 0xc5, 0xa5, 
            0x56, 0x5d, 0xb9, 0x28, 
            0x41, 0x4c, 0x66, 0xd1, 
            0x91, 0x2b, 0x11, 0xac, 
            0xf9, 0x74, 0xb2, 0xe6, 
            0x44, 0xca, 0xaa, 0xc0, 
            0x47, 0x39, 0xce, 0x99
        ])),
        27 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x2e, 0xab, 0x55, 0xf6,
            0xae, 0x4e, 0x66, 0xe3,
            0x2c, 0x51, 0x89, 0xee,
            0xd5, 0xc4, 0x70, 0x84,
            0x38, 0x44, 0x57, 0x60,
            0xf5, 0xed, 0x7e, 0x7b,
            0x69, 0xb2, 0xa6, 0x26,
            0x00, 0xf3, 0x54, 0x00
        ])),
        28 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x0, 0x2d, 0xf3, 0x7a, 
            0x26, 0x42, 0x62, 0x18, 
            0x2, 0x38, 0x3c, 0xf9, 
            0x52, 0xbf, 0x4d, 0xd1, 
            0xf3, 0x2e, 0x5, 0x43, 
            0x3b, 0xee, 0xb1, 0xfd, 
            0x41, 0x3, 0x1f, 0xb7, 
            0xea, 0xce, 0x97, 0x9d
        ])),
        29 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x10, 0x4a, 0xeb, 0x41, 
            0x43, 0x5d, 0xb6, 0x6c, 
            0x3e, 0x62, 0xfe, 0xcc, 
            0xc1, 0xd6, 0xf5, 0xd9, 
            0x8d, 0xa, 0xe, 0xd7, 
            0x5d, 0x13, 0x74, 0xdb, 
            0x45, 0x7c, 0xf4, 0x62, 
            0xe3, 0xa1, 0xf4, 0x27
        ])),
        30 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x1f, 0x3c, 0x6f, 0xd8, 
            0x58, 0xe9, 0xa7, 0xd4, 
            0xb0, 0xd1, 0xf3, 0x8e, 
            0x25, 0x6a, 0x9, 0xd8, 
            0x1d, 0x5a, 0x5e, 0x3c, 
            0x96, 0x39, 0x87, 0xe2, 
            0xd4, 0xb8, 0x14, 0xcf, 
            0xab, 0x7c, 0x6e, 0xbb
        ])),
        31 => Uint256::new(utils::vec_to_u128(
            &vec![
            0x2c, 0x7a, 0x07, 0xd2,
            0xe7, 0xe0, 0x47, 0x90,
            0x67, 0x0c, 0xd0, 0xc5,
            0x2b, 0x0c, 0x8f, 0x67,
            0x29, 0xc9, 0x32, 0x5f,
            0x98, 0x2b, 0xb5, 0x8a,
            0x89, 0xda, 0xfd, 0xc1,
            0x07, 0xea, 0x2f, 0x12
        ])),
        _ => panic!("Index out of bounds")
    }
}
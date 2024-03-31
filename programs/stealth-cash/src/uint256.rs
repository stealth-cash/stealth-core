use std::str::FromStr;
use anchor_lang::prelude::*;
use primitive_types::U256;
use hex;
use crate::utils;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialOrd)]
pub struct Uint256 {
    pub v: U256
}

impl AnchorSerialize for Uint256 {
    fn serialize<W: anchor_lang::prelude::borsh::maybestd::io::Write>(&self, writer: &mut W) -> anchor_lang::prelude::borsh::maybestd::io::Result<()> {
        let mut bytes = [0; 32];
        self.v.to_big_endian(&mut bytes);
        writer.write_all(&bytes).unwrap();
        Ok(())
    }
}

impl AnchorDeserialize for Uint256 {
    fn deserialize_reader<R: anchor_lang::prelude::borsh::maybestd::io::Read>(reader: &mut R) -> anchor_lang::prelude::borsh::maybestd::io::Result<Self> {
        let mut buf = [0; 32];
        reader.read_exact(&mut buf)?;
        Ok(Uint256 {
            v: U256::from(&buf)
        })
    }
}

impl FromStr for Uint256 {
    type Err = anchor_lang::error::Error;
    fn from_str(s: &str) -> Result<Self> {
        match U256::from_str_radix(s, 16) {
            Ok(n) => return Ok(Self { v: n }),
            Err(_) => return Err(AnchorError {
                error_name: "Parse Error".to_string(),
                error_code_number: 0,
                error_msg: "Failed to parse".to_string(),
                error_origin: None,
                compared_values: None
            }.into())
        }
    }
}

impl ToString for Uint256 {
    fn to_string(&self) -> String {
        let mut bytes = [0; 32];
        self.v.to_big_endian(&mut bytes);
        hex::encode(bytes)
    }
}

impl PartialEq for Uint256 {
    fn eq(&self, other: &Self) -> bool {
        self.v == other.v
    }
    
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Uint256 {
    pub fn new(n: u128) -> Self {
        Self { v: U256::from(n) }
    }

    pub fn from(s: &'static str) -> Self {
        Self { v: U256::from(s) }
    }

    pub fn from_string(s: &String) -> Self {
        Self { v: U256::from(s.as_bytes()) }
    }

    pub fn from_bytes(&self, bytes: &[u8]) -> Self {
        assert!(bytes.len() <= 32, "big-endian");
        return Self { v: U256::from_big_endian(bytes) }
    }

    pub fn to_bytes(&self, r: &mut [u8]) {
        self.v.to_big_endian(r);
    }

    pub fn zero() -> Self {
        Self::from_str("0x0").unwrap()
    }

    pub fn one() -> Self {
        Self::from_str("0x1").unwrap()
    }

    pub fn add_mod(&self, b: &Uint256, p: &Uint256) -> Uint256 {
        /* (a + b) mod p = [(a mod p) + (b mod p)] mod p */
        
        // a mod p
        let x1 = self.v.checked_rem(p.v).expect("modulo");
        
        // b mod p
        let x2 = b.v.checked_rem(p.v).expect("modulo");        
        
        let (mut x3, overflow) = x1.overflowing_add(x2);

        if overflow {
            x3 = x3
                .checked_add(
                    U256::MAX.checked_sub(p.v).expect("sub")
                        .checked_add(U256::from_big_endian(&[1])).expect("conversion")   
                ).expect("conversion")
        }

        x3 = x3.checked_rem(p.v).expect("modulo");

        return Uint256 { v: x3 };
    }

    pub fn sub_mod(&self, b: &Uint256, p: &Uint256) -> Uint256 {
        /* 
            (a - b) mod p 
            => [(a mod p) - (b mod p)] mod p 
            => [a mod p + (p - b) mod p] mod p 
        */
        let x1 = self.v.checked_rem(p.v).expect("modulo");
        let x2 = b.v.checked_rem(p.v).expect("modulo");

        return Uint256 { v: x1 }.add_mod(&Uint256 { v: p.v - x2 }, p);
    }

    pub fn mul_mod(&self, b: &Uint256, p: &Uint256) -> Uint256 {
        /*
            add-and-double / square-and-multiply
            9 * 2 = 2 + 2 ... + 2;
            
            9 = b'1001
            
            iterate through b'1001, base = 0
                if n = 1:
                    base *= x
                else:
                    base *= x
                    base += x 

            base = 0
            base = 2
            base = 4
            base = 8
            base = 18
            
        */
        let x1 = Self { v: self.v.checked_rem(p.v).expect("modulo") } ;
        let x2 = Self { v: b.v.checked_rem(p.v).expect("modulo") };
        
        let mut base = Self::zero();
        
        let seq: Self;
        let adder: Self;

        if x1.v < x2.v {
            seq = x1.clone();
            adder = x2.clone();
        } else {
            seq = x2.clone();
            adder = x1.clone();
        }

        let mut seq_bytes = [0; 32];
        seq.to_bytes(&mut seq_bytes);
        let mut seq_binaries: Vec<u8> = vec![];
        
        utils::bytes_to_binary(&seq_bytes, &mut seq_binaries);

        let mut on = false;
        for d in seq_binaries.into_iter() {
            if on {
                base = base.add_mod(&base, p);
            }
            if d > 0 {
                on = true;
                base = base.add_mod(&adder, p);
            }
        }

        return base;
    }

    pub fn exp_mod(&self, e: &Uint256, p: &Uint256) -> Uint256 {
        let seq = e.clone();
        let multiplier = Self { v: self.v.checked_rem(p.v).expect("modulo") };

        let mut base = Self::one();

        let mut seq_bytes = [0; 32];
        seq.to_bytes(&mut seq_bytes);

        let mut seq_binaries: Vec<u8> = vec![];
        utils::bytes_to_binary(&seq_bytes, &mut seq_binaries);

        let mut on = false;
        for d in seq_binaries.into_iter() {
            if on {
                base = base.mul_mod(&base, p);
            }
            if d > 0 {
                on = true;
                base = base.mul_mod(&multiplier, p);
            }
        }

        return base;
    }

    pub fn div_mod(&self, b: &Uint256, p: &Uint256) -> Uint256 {
        return self.mul_mod(&b.exp_mod(&Self{ v: p.v - 2 }, p), p);
    }

    pub fn is_zero(&self) -> bool {
        self.v.is_zero()
    }
}
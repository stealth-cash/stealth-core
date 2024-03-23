#[cfg(not(target_os = "solana"))]
use rayon::{prelude::*, iter::{IntoParallelIterator,ParallelIterator}};
use crate::{HashingAlgorithm, MerkleError};
use anchor_lang::prelude::{borsh::{BorshDeserialize, BorshSerialize}, *};
use super::MerkleProof;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct MerkleTree {
    algorithm: HashingAlgorithm,
    hash_size: u8,
    root: Vec<u8>,
    hashes: Vec<Vec<Vec<u8>>>
}

// For non-Solana targets, use Rayon to hash/merklize in parallel
#[cfg(not(target_os = "solana"))]
impl MerkleTree {
    fn merklize_unchecked(h: &Vec<Vec<u8>>, a: HashingAlgorithm, s: usize) -> Vec<Vec<u8>> {
        h.par_chunks(2).into_par_iter().map(|h| {
            if h.len() > 1 {
                a.hash(&vec![h[0].clone(),h[1].clone()].concat(), s)
            } else {
                a.hash(&vec![h[0].clone(),h[0].clone()].concat(), s)
            }
        }).collect()
    }

    fn add_leaves(&mut self, leaves: &Vec<Vec<u8>>) -> Result<()> {
        let hashes: Vec<Vec<u8>> = leaves.into_par_iter().map(|leaf| {
            self.double_hash(leaf)
        }).collect();
        self.add_hashes_unchecked(hashes)
    }
}

// For Solana targets, merklize in serial
#[cfg(target_os = "solana")]
impl MerkleTree {
    fn merklize_unchecked(h: &Vec<Vec<u8>>, a: HashingAlgorithm, s: usize) -> Vec<Vec<u8>> {
        h.chunks(2).into_iter().map(|h| {
            if h.len() > 1 {
                a.hash(&vec![h[0].clone(),h[1].clone()].concat(), s)
            } else {
                a.hash(&vec![h[0].clone(),h[0].clone()].concat(), s)
            }
        }).collect()
    }

    fn add_leaves(&mut self, leaves: &Vec<Vec<u8>>) -> Result<()> {
        let hashes: Vec<Vec<u8>> = leaves.into_iter().map(|leaf| {
            self.double_hash(leaf)
        }).collect();
        self.add_hashes_unchecked(hashes)
    }
}

impl MerkleTree {
    // Initialize a new tree with configurable size and hashing params
    pub fn new(algorithm: HashingAlgorithm, hash_size: u8) -> Self {
        let mut hash_size = hash_size;
        if hash_size == 0 || hash_size > 32 {
            hash_size = 32
        }
        Self {
            algorithm,
            root: vec![],
            hash_size,
            hashes: vec![vec![]]
        }
    }

    // Append multiple hashes with a length check. Use with unnormalized data
    pub fn add_hashes(&mut self, hashes: Vec<Vec<u8>>) -> Result<()> {
        for hash in hashes.iter() {
            if hash.len() != self.hash_size as usize {
                return Err(MerkleError::InvalidHashSize.into());
            }
        }
        self.hashes[0].extend_from_slice(&hashes);
        Ok(())
    }

    // Append multiple hashes without a length check. Use with normalized data
    pub fn add_hashes_unchecked(&mut self, hashes: Vec<Vec<u8>>) -> Result<()> {
        self.hashes[0].extend_from_slice(&hashes);
        Ok(())
    }

    // Hash with defined hashing algorithm and truncate to defined length
    fn hash(&self, m: &[u8]) -> Vec<u8> {
        self.algorithm.hash(m, self.hash_size as usize)
    }

    // Double hash with defined hashing algorithm and truncate to defined length
    fn double_hash(&self, m: &[u8]) -> Vec<u8> {
        self.algorithm.double_hash(m, self.hash_size as usize)
    }

    // Hash and append a leaf
    pub fn add_leaf(&mut self, leaf: &[u8]) -> usize {
        // Double hash to prevent length extension attacks
        // No need for length check
        self.add_hash_unchecked(self.double_hash(leaf))
    }

    // Append a hash with a length check. Use with unnormalized data
    pub fn add_hash(&mut self, hash: Vec<u8>) -> Result<()> {
        if hash.len() != self.hash_size as usize {
            return Err(MerkleError::InvalidHashSize.into())
        }
        self.add_hash_unchecked(hash);
        Ok(())
    }

    // Append a hash without a length check. Use with normalized data
    pub fn add_hash_unchecked(&mut self, hash: Vec<u8>) -> usize {
        self.hashes[0].push(hash);
        self.hashes[0].len() - 1
    }

    pub fn merklize(&mut self) -> Result<()> {
        let len = self.hashes[0].len();
        match len {
            0 => Err(MerkleError::TreeEmpty.into()),
            1 => {
                self.reset();
                self.root = self.hashes[0][0].clone();
                Ok(())
            }, 
            _ => {
                self.reset();
                let mut count = self.hashes[0].len();
                while count > 2 {
                    let h: Vec<Vec<u8>> = Self::merklize_unchecked(self.hashes.last().ok_or(MerkleError::BranchOutOfRange)?, self.algorithm.clone(), self.hash_size as usize);
                    count = h.len();
                    self.hashes.push(h);
                }
                self.root = Self::merklize_unchecked(self.hashes.last().ok_or(MerkleError::BranchOutOfRange)?, self.algorithm.clone(), 32 as usize)[0].clone();
                Ok(())
            }
        }
    }

    pub fn reset(&mut self) {
        self.hashes.truncate(1);
    }

    fn merklized(&self) -> Result<()> {
        if self.root.eq(&[0u8;32]) {
            return Err(MerkleError::TreeNotMerklized.into())
        }
        Ok(())
    }

    fn within_range(&self, index: usize) -> Result<()> {
        let len = self.hashes[0].len();
        if index > len {
            return Err(MerkleError::LeafOutOfRange.into())
        }
        Ok(())
    }

    fn get_hash_index(&self, hash: Vec<u8>) -> Result<usize> {
        match self.hashes[0].binary_search(&hash) {
            Ok(i) => Ok(i),
            Err(_) => Err(MerkleError::LeafNotFound.into())
        }
    }

    pub fn get_merkle_root(&self) -> Result<Vec<u8>> {
        self.merklized()?;
        Ok(self.root.clone())
    }

    pub fn get_leaf_hash(&self, i: usize) -> Result<Vec<u8>> {
        self.within_range(i)?;
        Ok(self.hashes[0][i].clone())
    }

    pub fn merkle_proof_hash(&self, hash: Vec<u8>) -> Result<MerkleProof> {
        self.merklized()?;
        let i = self.get_hash_index(hash)?;
        self.merkle_proof_index_unchecked(i)
    }

    pub fn merkle_proof_index(&self, i: usize) -> Result<MerkleProof> {
        self.merklized()?;
        self.within_range(i)?;
        self.merkle_proof_index_unchecked(i)
    }

    fn merkle_proof_index_unchecked(&self, i: usize) -> Result<MerkleProof> {
        let len = self.hashes[0].len();
        match len {
            // We can't have zero leaves in a Merkle tree
            0 => Err(MerkleError::TreeEmpty.into()),
            // If we only have one leaf, the 0th hash is the root
            1 => Ok(MerkleProof::new(
                self.algorithm.clone(),
                self.hash_size,
                i as u32,
                vec![],
            )),
            _ => {
                let mut hashes: Vec<Vec<u8>> = vec![];
                let mut n = i;
                // 0, 1, 2, 3
                for x in 0..self.hashes.len() {
                    n = match n%2 == 0 {
                        true => usize::min(n+1, self.hashes[x].len()),
                        false => n-1
                    };
                    
                    match self.hashes[x].get(n) {
                        Some(h) => {
                            hashes.push(h.clone())
                        },
                        None => hashes.push(self.hashes[x][n-1].clone())
                    }
                    n = n.saturating_div(2);
                }
                Ok(MerkleProof::new(
                    self.algorithm.clone(),
                    self.hash_size,
                    i as u32,
                    hashes.concat()
                ))
            }
        }
    }
}
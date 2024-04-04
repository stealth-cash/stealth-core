use std::collections::HashMap;
use anchor_lang::prelude::*;

declare_id!("5Ta8DofvfQ8FoJvwjApYe7jbXqqwT4UpXrBXBX3eTVxz");

pub mod merkle_tree;
pub mod utils;
// pub mod verifier;
pub mod hasher;
pub mod uint256;

use merkle_tree::*;
use uint256::Uint256;
use utils::*;

#[program]
pub mod stealth_cash {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        // verifier: Pubkey,
        denomination: u64,
        merkle_tree_height: u8
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        // state.verifier = verifier;
        state.denomination = denomination;
        state.merkle_tree = MerkleTree::new(merkle_tree_height).to_string();
        state.commitments = String::new();
        state.nullifier_hashes = String::new();
        Ok(())
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        _commitment: String // Uin256
    ) -> Result<DepositEvent> {
        let serialized_state = &ctx.accounts.state;
        let mut state = serialized_state.deserialize();

        let commitment = Uint256::from_string(&_commitment);

        if state.commitments.get(&commitment).is_some() {
            return Err(err("Commitment is submitted").into());
        }

        let leaf_index = state.merkle_tree.insert(commitment.clone()).unwrap() as u32;
        let timestamp: i64 = Clock::get().unwrap().unix_timestamp;
        state.nullifier_hashes.insert(commitment.clone(), true);

        let deposit_event = DepositEvent {
            commitment: commitment.to_string(), // TODO
            leaf_index,
            timestamp
        };

        process_deposit();

        Ok(deposit_event)
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        _proof: String, // Uint256
        _root: String, // Uint256
        _nullifier_hash: String, // Uint256
        _recipient: Pubkey,
        _relayer: Pubkey,
        _fee: f64,
        _refund: f64
    ) -> Result<WithdrawalEvent> {
        let serialized_state = &mut ctx.accounts.state;
        let mut state = serialized_state.deserialize();

        let proof = Uint256::from_string(&_proof);
        let nullifier_hash = Uint256::from_string(&_nullifier_hash);
        let root = Uint256::from_string(&_root);

        if _fee > state.denomination as f64 {
            return Err(err("Fee exceeds denomination").into());
        }

        if state.nullifier_hashes.get(&nullifier_hash).is_some() {
            return Err(err("The note has already been spent").into());
        }

        if !state.merkle_tree.is_known_root(root.clone()) {
            return Err(err("Could not find merkle root").into());
        }
    
        let tuple: (Uint256, Uint256, u128, u128, f64, f64) = (
            root, 
            nullifier_hash, 
            pubkey_to_u128(&_recipient), 
            pubkey_to_u128(&_relayer),
            _fee,
            _refund
        );
        if !verify_proof(proof, tuple) {
            return Err(err("Invalid withdraw proof").into());
        }

        state.nullifier_hashes.insert(nullifier_hash.clone(), true);
        process_withdraw(&_recipient, &_relayer, _fee, _refund);

        let withdrawal_event = WithdrawalEvent {
            to: _recipient,
            nullifier_hash: _nullifier_hash.to_string(), // TODO
            relayer: _relayer,
            fee: _fee
        };
        
        Ok(withdrawal_event)
    }

}

fn process_deposit() {
    unimplemented!()
}

fn process_withdraw(_recipient: &Pubkey, _relayer: &Pubkey, _fee: f64, _refund: f64) {
    unimplemented!()
}

/**************
    Data Transfer Accounts
**************/

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    state: Account<'info, State>
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    state: Account<'info, State>,
    
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    sender: AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    recipient: AccountInfo<'info>,

    system_program: SystemAccount<'info>
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    state: Account<'info, State>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(signer)]
    authority: AccountInfo<'info>
}

/**************
    Events
**************/

#[event]
pub struct DepositEvent {
    commitment: String,
    leaf_index: u32,
    timestamp: i64
}

#[event]
pub struct WithdrawalEvent {
    to: Pubkey,
    nullifier_hash: String, //Uint256,
    relayer: Pubkey,
    fee: f64
}

/**************
    Contract State Account
**************/

#[account]
pub struct State {
    pub verifier: Pubkey,
    pub denomination: u64,
    pub merkle_tree: String, //MerkleTree,
    pub commitments: String, //HashMap<Uint256, bool>,
    pub nullifier_hashes: String //HashMap<Uint256, bool>
}

pub struct DeserializedState {
    pub verifier: Pubkey,
    pub denomination: u64,
    pub merkle_tree: MerkleTree,
    pub commitments: HashMap<Uint256, bool>,
    pub nullifier_hashes: HashMap<Uint256, bool>
}

impl State {
    pub fn deserialize(&self) -> DeserializedState {
        DeserializedState {
            verifier: self.verifier,
            denomination: self.denomination,
            merkle_tree: self.merkle_tree.parse().unwrap(),
            commitments: DeserializedState::deserialize_map(&self.commitments),
            nullifier_hashes: DeserializedState::deserialize_map(&self.nullifier_hashes),
        }
    }
}

impl DeserializedState {
    fn deserialize_map(serialized_map: &str) -> HashMap<Uint256, bool> {
        let bytes = serialized_map.as_bytes();
        if let Ok(map) = HashMap::try_from_slice(&bytes) {
            map
        } else {
            HashMap::new()
        }
    }
}

fn verify_proof(_proof: Uint256, _input: (Uint256, Uint256, u128, u128, f64, f64)) -> bool {
    true
}

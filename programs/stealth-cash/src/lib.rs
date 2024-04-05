use std::collections::HashMap;
use anchor_lang::prelude::*;

declare_id!("GZFcqq4j4ptgHMVnFk8t4hDxCRS5Rrt1aNCBNj4hX3Lt");

use stealth_lib::{merkle_tree::MerkleTree, uint256::Uint256};

pub mod helpers;
use helpers::{anchor_err, pubkey_to_u128};

#[program]
pub mod stealth_cash {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        denomination: u64,
        merkle_tree_height: u8
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        let deserialized = DeserializedState {
            denomination,
            merkle_tree: MerkleTree::new(merkle_tree_height),
            commitments: HashMap::new(),
            nullifier_hashes: HashMap::new()
        };
        let serialized = deserialized.serialize();
        state.denomination = serialized.denomination;
        state.merkle_tree = serialized.merkle_tree;
        state.commitments = serialized.commitments;
        state.nullifier_hashes = serialized.nullifier_hashes;
        Ok(())
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        _commitment: String // Uint256
    ) -> Result<DepositEvent> {
        msg!("Depositing");
        let state = &mut ctx.accounts.state;
        let mut deserialized_state = state.deserialize();

        let commitment = Uint256::from_string(&_commitment);

        if deserialized_state.commitments.get(&commitment).is_some() {
            return Err(anchor_err("Commitment is submitted").into());
        }

        let leaf_index = deserialized_state.merkle_tree.insert(commitment.clone()).unwrap() as u32;
        let timestamp: i64 = Clock::get().unwrap().unix_timestamp;
        deserialized_state.nullifier_hashes.insert(commitment.clone(), true);

        let serialized = deserialized_state.serialize();
        state.commitments = serialized.commitments;
        state.denomination = serialized.denomination;
        state.merkle_tree = serialized.merkle_tree;
        state.nullifier_hashes = serialized.nullifier_hashes;

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
            return Err(anchor_err("Fee exceeds denomination").into());
        }

        if state.nullifier_hashes.get(&nullifier_hash).is_some() {
            return Err(anchor_err("The note has already been spent").into());
        }

        if !state.merkle_tree.is_known_root(root.clone()) {
            return Err(anchor_err("Could not find merkle root").into());
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
            return Err(anchor_err("Invalid withdraw proof").into());
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
    // The account paying to create the counter account
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(init_if_needed, space = 10000, payer = payer)]
    pub state: Account<'info, State>,
    
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub sender: AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub recipient: AccountInfo<'info>,

    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(signer)]
    pub authority: AccountInfo<'info>
}

/**************
    Events
**************/

#[event]
pub struct DepositEvent {
    pub commitment: String,
    pub leaf_index: u32,
    pub timestamp: i64
}

#[event]
pub struct WithdrawalEvent {
    pub to: Pubkey,
    pub nullifier_hash: String, //Uint256,
    pub relayer: Pubkey,
    pub fee: f64
}

/**************
    Contract State Account
**************/

#[account]
pub struct State {
    pub denomination: u64,
    pub merkle_tree: String, //MerkleTree,
    pub commitments: String, //HashMap<Uint256, bool>,
    pub nullifier_hashes: String //HashMap<Uint256, bool>
}

pub struct DeserializedState {
    pub denomination: u64,
    pub merkle_tree: MerkleTree,
    pub commitments: HashMap<Uint256, bool>,
    pub nullifier_hashes: HashMap<Uint256, bool>
}

impl State {
    pub fn deserialize(&self) -> DeserializedState {
        DeserializedState {
            denomination: self.denomination,
            merkle_tree: self.merkle_tree.parse().unwrap(),
            commitments: DeserializedState::deserialize_map(&self.commitments),
            nullifier_hashes: DeserializedState::deserialize_map(&self.nullifier_hashes),
        }
    }
}

impl DeserializedState {
    pub fn serialize(&self) -> State {
        State {
            denomination: self.denomination,
            merkle_tree: self.merkle_tree.to_string(),
            commitments: DeserializedState::serialize_map(&self.commitments),
            nullifier_hashes: DeserializedState::serialize_map(&self.nullifier_hashes)
        }
    }

    fn serialize_map(map: &HashMap<Uint256, bool>) -> String {
        let mut result = String::new();
        for (key, value) in map {
            result.push_str(&format!("{}:{};", key.to_string(), value));
        }
        result
    }

    pub fn deserialize_map(serialized_map: &str) -> HashMap<Uint256, bool> {
        let mut map = HashMap::new();
        for pair in serialized_map.split(';') {
            let mut split = pair.split(':');
            let key = Uint256::from_string(&split.next().unwrap().to_string());
            let value = split.next().unwrap().parse().unwrap();
            map.insert(key, value);
        }
        map
    }
}

fn verify_proof(_proof: Uint256, _input: (Uint256, Uint256, u128, u128, f64, f64)) -> bool {
    true
}

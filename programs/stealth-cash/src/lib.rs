// TODO: CHANGE HASHMAPS TO ARRAYS
// TODO: CHANGE HASHMAPS TO ARRAYS
// TODO: CHANGE HASHMAPS TO ARRAYS
// TODO: CHANGE HASHMAPS TO ARRAYS
// TODO: CHANGE HASHMAPS TO ARRAYS

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
        verifier: Pubkey,
        denomination: u64,
        merkle_tree_height: u8
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.verifier = verifier;
        state.denomination = denomination;
        state.merkle_tree = MerkleTree::new(merkle_tree_height);
        state.commitments = HashMap::new();
        state.nullifier_hashes = HashMap::new();
        Ok(())
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        _commitment: Uint256
    ) -> Result<DepositEvent> {
        let state = &mut ctx.accounts.state;

        if state.commitments.get(&_commitment).is_some() {
            let e = AnchorError {
                error_msg: "Commitment is submitted".to_string(),
                error_name: "CommitmentSubmittedException".to_string(),
                error_code_number: 0,
                error_origin: None,
                compared_values: None
            };
            return Err(e.into());
        }

        let leaf_index = state.merkle_tree.insert(_commitment.clone()).unwrap() as u32;
        let timestamp: i64 = Clock::get().unwrap().unix_timestamp;
        state.nullifier_hashes.insert(_commitment.clone(), true);

        let deposit_event = DepositEvent {
            commitment: _commitment,
            leaf_index,
            timestamp
        };

        process_deposit();

        Ok(deposit_event)
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        _proof: Uint256,
        _root: Uint256,
        _nullifier_hash: Uint256,
        _recipient: Pubkey,
        _relayer: Pubkey,
        _fee: f64,
        _refund: f64
    ) -> Result<WithdrawalEvent> {
        let state = &mut ctx.accounts.state;

        if _fee > state.denomination as f64 {
            let e = AnchorError {
                error_msg: "Fee exceeds denomination".to_string(),
                error_name: "FeeExceedsDenominationException".to_string(),
                error_code_number: 0,
                error_origin: None,
                compared_values: None
            };
            return Err(e.into());
        }

        if state.nullifier_hashes.get(&_nullifier_hash).is_some() {
            let e = AnchorError {
                error_msg: "The note has already been spent".to_string(),
                error_name: "DuplicateNullifierHashException".to_string(),
                error_code_number: 0,
                error_origin: None,
                compared_values: None
            };
            return Err(e.into());
        }

        if !state.merkle_tree.is_known_root(_root.clone()) {
            let e = AnchorError {
                error_msg: "Could not find merkle root".to_string(),
                error_name: "MerkleRootNotFoundExcepion".to_string(),
                error_code_number: 0,
                error_origin: None,
                compared_values: None
            };
            return Err(e.into());
        }
    
        let tuple: (Uint256, Uint256, u128, u128, f64, f64) = (
            _root, 
            _nullifier_hash, 
            pubkey_to_u128(&_recipient), 
            pubkey_to_u128(&_relayer),
            _fee,
            _refund
        );
        if !verify_proof(_proof, tuple) {
            let e = AnchorError {
                error_msg: "Invalid withdraw proof".to_string(),
                error_name: "InvalidWithdrawProofException".to_string(),
                error_code_number: 0,
                error_origin: None,
                compared_values: None
            };
            return Err(e.into());
        }

        state.nullifier_hashes.insert(_nullifier_hash.clone(), true);
        process_withdraw(&_recipient, &_relayer, _fee, _refund);

        let withdrawal_event = WithdrawalEvent {
            to: _recipient,
            nullifier_hash: _nullifier_hash,
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
    commitment: Uint256,
    leaf_index: u32,
    timestamp: i64
}

#[event]
pub struct WithdrawalEvent {
    to: Pubkey,
    nullifier_hash: Uint256,
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
    pub merkle_tree: MerkleTree,
    pub commitments: HashMap<Uint256, bool>,
    pub nullifier_hashes: HashMap<Uint256, bool>
}
use std::collections::HashMap;

use anchor_lang::prelude::{borsh::BorshSerialize, *};

declare_id!("5Ta8DofvfQ8FoJvwjApYe7jbXqqwT4UpXrBXBX3eTVxz");

pub mod merkle_tree;
pub mod utils;

use merkle_tree::*;
use utils::*;

type Commitment = Vec<u8>;

#[program]
pub mod stealth_cash {

    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        _verifier: Pubkey,
        _hasher: Pubkey,
        _denomination: u64,
        _merkle_tree_height: u32
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.verifier = _verifier;
        state.denomination = _denomination;
        state.merkle_tree = MerkleTree::new(32);
        Ok(())
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        _commitment: Commitment
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
        _proof: Commitment,
        _root: Commitment,
        _nullifier_hash: Commitment,
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
    
        let tuple: (u128, u128, u128, u128, f64, f64) = (
            vec_to_u128(&_root), 
            vec_to_u128(&_nullifier_hash), 
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

fn process_withdraw(recipient: &Pubkey, relayer: &Pubkey, fee: f64, refund: f64) {
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
    
    #[account(signer)]
    authority: AccountInfo<'info>,

    system_program: AccountInfo<'info>
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    state: Account<'info, State>,

    #[account(signer)]
    authority: AccountInfo<'info>
}


/**************
    Events
**************/

#[event]
pub struct DepositEvent {
    commitment: Commitment,
    leaf_index: u32,
    timestamp: i64
}

#[event]
pub struct WithdrawalEvent {
    to: Pubkey,
    nullifier_hash: Commitment,
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
    pub commitments: HashMap<Commitment, bool>,
    pub nullifier_hashes: HashMap<Commitment, bool>
}


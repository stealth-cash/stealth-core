use std::collections::HashMap;

use anchor_lang::prelude::{borsh::BorshSerialize, *};

declare_id!("5Ta8DofvfQ8FoJvwjApYe7jbXqqwT4UpXrBXBX3eTVxz");

use svm_merkle_tree::{self, MerkleTree};

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
        state.merkle_tree = MerkleTree::new(svm_merkle_tree::HashingAlgorithm::Sha256, 32);
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, _commitment: Commitment) -> Result<DepositEvent> {
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

        let leaf_index = state.merkle_tree.add_hash_unchecked(_commitment.clone()) as u32;
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
        _relayer: Option<Pubkey>,
        _fee: f64
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

        // if state.merkle_tree.is_known_root()


        todo!()
    }

}

fn process_deposit() {
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
    relayer: Option<Pubkey>,
    fee: u64
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


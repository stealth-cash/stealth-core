use anchor_lang::prelude::{borsh::{BorshDeserialize, BorshSerialize}, *};

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

    pub fn deposit(ctx: Context<Deposit>, _commitment: Commitment) -> Result<()> {
        let state = &mut ctx.accounts.state;

        if state.commitments.iter().any(|c| *c == _commitment) {
            return Err(ErrorCode::InstructionMissing.into());
        }

        Ok(())
    }
}

/*
    Data Transfer Accounts 
*/

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

/*
    Events
*/

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

/**
 * Contract State Account
 */

#[account]
pub struct State {
    pub verifier: Pubkey,
    pub denomination: u64,
    pub merkle_tree: MerkleTree,
    pub commitments: Vec<Commitment>,
    pub nullifier_hashes: Vec<Commitment>
}


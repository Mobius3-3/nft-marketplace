use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::state::marketplace::Marketplace;

/// Accounts required for initializing a new marketplace
#[derive(Accounts)]
#[instruction(name: String)]
pub struct Initialize<'info>{
    #[account(mut)]
    pub admin: Signer<'info>, // The signer who will be the marketplace admin
    
    #[account(
        init,
        payer = admin,
        seeds = [b"marketplace", name.as_str().as_bytes()],
        bump,
        space = Marketplace::INIT_SPACE,
    )]
    pub marketplace: Account<'info, Marketplace>, // Main marketplace PDA derived from name
    
    #[account(
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump,
    )]
    pub treasury: SystemAccount<'info>, // Treasury PDA to collect marketplace fees
    
    #[account(
        init,
        payer = admin,
        seeds = [b"rewards", marketplace.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = marketplace,
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>, // Reward token mint for the marketplace

    pub system_program: Program<'info, System>, // Required for creating accounts
    pub token_program: Interface<'info, TokenInterface>, // Required for token operations
}

impl <'info> Initialize<'info> {
    // Initialize the marketplace with provided configuration
    pub fn init(&mut self, name: String, fee: u16, bumps: &InitializeBumps) -> Result<()>{
        // Set marketplace account data
        self.marketplace.set_inner(Marketplace{
            admin: self.admin.key(),
            fee,
            bump: bumps.marketplace,
            treasury_bump: bumps.treasury,
            rewards_bump: bumps.reward_mint,
            name,
        });

        Ok(())
    }
}

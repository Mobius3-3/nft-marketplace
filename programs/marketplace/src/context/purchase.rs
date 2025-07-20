use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use anchor_spl::{associated_token::AssociatedToken, metadata::{mpl_token_metadata::types::SeedsVec, MasterEditionAccount, Metadata, MetadataAccount}, token::{close_account, mint_to, transfer_checked, CloseAccount, MintTo, TransferChecked}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::state::{Listing, Marketplace};

#[derive(Accounts)]
pub struct Purchase<'info>{
    #[account(mut)]
    pub taker: Signer<'info>, 
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    pub maker_mint: InterfaceAccount<'info, Mint>, // The NFT mint being listed

    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>, // The marketplace configuration account
    
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = maker_mint,
        associated_token::authority = taker,
    )]
    pub taker_ata: InterfaceAccount<'info, TokenAccount>, // Token account holding the NFT

    #[account(
        mut,
        associated_token::mint = maker_mint,
        associated_token::authority = listing,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>, // Escrow account for the NFT during listing

    #[account(
        mut,
        seeds = [marketplace.key().as_ref(), maker_mint.key().as_ref()],
        bump = listing.bump
    )]
    pub listing: Account<'info, Listing>, // Account to store listing information

    #[account(
      mut,
      seeds = [b"rewards", marketplace.key().as_ref()],
      bump,
      mint::decimals = 6,
      mint::authority = marketplace,
    )]
    pub rewards: InterfaceAccount<'info, Mint>,

    #[account(
      seeds = [b"treasury", marketplace.key().as_ref()],
      bump,
    )]
    pub treasury: SystemAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>, // For creating ATAs
    pub system_program: Program<'info, System>, // For creating accounts
    pub token_program: Interface<'info, TokenInterface> // For token operations
}

impl<'info> Purchase<'info> {
  pub fn send_sol(&self) -> Result<()> {
    let cpi_accounts = Transfer {
      from: self.taker.to_account_info(),
      to: self.maker.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), cpi_accounts);
    
    let amount = self.listing.price - self.listing.price.checked_mul(self.marketplace.fee as u64).unwrap().checked_div(10000).unwrap();
    transfer(cpi_ctx, amount)
  }

  pub fn receive_nft(&self) -> Result<()> {
    let seeds = &[
      &self.marketplace.key().to_bytes()[..], 
      &self.maker_mint.key().to_bytes()[..],
      &[self.listing.bump]
    ];
    let signers_seeds = &[&seeds[..]];

    let cpi_program = self.token_program.to_account_info();

    let cpi_accounts = TransferChecked{
        to: self.taker_ata.to_account_info(), // Source of the NFT
        mint: self.maker_mint.to_account_info(), // NFT mint 
        from: self.vault.to_account_info(), // Destination vault
        authority: self.listing.to_account_info(), // Authority to move the token
    };

    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers_seeds);

    transfer_checked(cpi_ctx, 1, self.maker_mint.decimals)?;

    Ok(())

  }

  // pub fn  receive_rewards(&self) -> Reuslt<()> {
  // todo!()

  // }

}
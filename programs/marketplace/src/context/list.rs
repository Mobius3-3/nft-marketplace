use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, metadata::{MasterEditionAccount, Metadata, MetadataAccount}, token::{transfer_checked, TransferChecked}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::state::{Listing, Marketplace};

#[derive(Accounts)]
pub struct List<'info>{
    #[account(mut)]
    pub maker: Signer<'info>, // The NFT owner creating the listing

    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>, // The marketplace configuration account

    pub maker_mint: InterfaceAccount<'info, Mint>, // The NFT mint being listed
    #[account(
        mut,
        associated_token::mint = maker_mint,
        associated_token::authority = maker,
    )]
    pub maker_ata: InterfaceAccount<'info, TokenAccount>, // Token account holding the NFT

    #[account(
        init,
        payer = maker,
        associated_token::mint = maker_mint,
        associated_token::authority = listing,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>, // Escrow account for the NFT during listing

    #[account(
        init,
        payer = maker,
        seeds = [marketplace.key().as_ref(), maker_mint.key().as_ref()],
        bump,
        space = 8 + Listing::INIT_SPACE,
    )]
    pub listing: Account<'info, Listing>, // Account to store listing information

    pub collection_mint: InterfaceAccount<'info, Mint>, // Collection the NFT belongs to
    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            maker_mint.key().as_ref(),
        ],
        seeds::program = metadata_program.key(),
        bump,
        constraint = metadata.collection.as_ref().unwrap().key.as_ref() == collection_mint.key().as_ref(),
        constraint = metadata.collection.as_ref().unwrap().verified == true,
    )]
    pub metadata: Account<'info, MetadataAccount>, // NFT metadata to verify collection
    
    #[account(
        seeds = [
            b"metadata", 
            metadata_program.key().as_ref(),
            maker_mint.key().as_ref(),
            b"edition"
        ],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub master_edition: Account<'info, MasterEditionAccount>, // Master edition to verify it's an NFT (one of a kind)

    
    pub metadata_program: Program<'info, Metadata>, // Metaplex program
    pub associated_token_program: Program<'info, AssociatedToken>, // For creating ATAs
    pub system_program: Program<'info, System>, // For creating accounts
    pub token_program: Interface<'info, TokenInterface> // For token operations
}

impl <'info> List<'info> {
    pub fn create_listing(&mut self, price: u64, bumps: &ListBumps) ->Result<()>{
        self.listing.set_inner(Listing{
            maker: self.maker.key(),
            maker_mint: self.maker_mint.key(),
            price,
            bump: bumps.listing,
        });

        Ok(())
    }

    pub fn deposit_nft(&mut self) ->Result<()>{
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked{
            from: self.maker_ata.to_account_info(), // Source of the NFT
            mint: self.maker_mint.to_account_info(), // NFT mint 
            to: self.vault.to_account_info(), // Destination vault
            authority: self.maker.to_account_info(), // Authority to move the token
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, 1, self.maker_mint.decimals)?;

        Ok(())
    }
}

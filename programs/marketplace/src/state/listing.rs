use anchor_lang::prelude::*;

#[account]
pub struct Listing{
    pub maker: Pubkey, // The wallet address of the seller who created this listing
    pub maker_mint: Pubkey, // The mint address of the NFT being sold
    pub price: u64, // The selling price in lamports (SOL's smallest unit)
    pub bump: u8,
}

// Implementation of the Space trait to define storage requirements
impl Space for Listing {
    /// Calculate the exact space needed for this account:
    /// - 8 bytes: Account discriminator (automatically added by Anchor)
    /// - 32 bytes: Pubkey for maker
    /// - 32 bytes: Pubkey for maker_mint
    /// - 8 bytes: u64 for price
    /// - 1 byte: u8 for bump
    const INIT_SPACE: usize = 8 + 32 + 32 + 8 + 1;
}

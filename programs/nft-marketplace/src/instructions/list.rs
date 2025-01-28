use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{ MasterEditionAccount, Metadata, MetadataAccount },
    token_interface::{ transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked },
};

use crate::{ errors::MarketplaceError, state::{ Listing, Marketplace } };

#[derive(Accounts)]
#[instruction(name: String)]
pub struct List<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(seeds = [b"marketplace", name.as_str().as_bytes()], bump = marketplace.bump)]
    pub marketplace: Box<Account<'info, Marketplace>>,

    //maker_mint represents the NFT
    pub maker_mint: Box<InterfaceAccount<'info, Mint>>,
    //collection_mint to make sure its not a fake NFT
    pub collection_mint: Box<InterfaceAccount<'info, Mint>>,
    //maker_ata represents the NFT ATA account, where it is stored
    #[account(
        mut,
        associated_token::mint = maker_mint,
        associated_token::authority = maker,
    )]
    pub maker_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init,
        payer = maker,
        space = Listing::INIT_SPACE,
        seeds = [marketplace.key().as_ref(), maker_mint.key().as_ref()],
        bump
    )]
    pub listing: Box<Account<'info, Listing>>,

    #[account(
        init_if_needed,
        payer = maker,
        seeds = [b"vault", maker_mint.key().as_ref()],
        bump,
        token::authority = listing,
        token::mint = maker_mint
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(), //metadata program id
            maker_mint.key().as_ref(),      //mint of the NFT both are predfined by the metadata program and need to be present in the seeds
        ],
        seeds::program = metadata_program.key(),  //doing seeds program and specifiying the metadata program we are saying we dont own it not writting it anchor under the hood passes the program id 
        bump,
        constraint = metadata.collection.as_ref().unwrap().key.as_ref() == collection_mint.key().as_ref() @MarketplaceError::CollectionInvalid,
        constraint = metadata.collection.as_ref().unwrap().verified == true @MarketplaceError::CollectionNotVerified,
    )]
    pub metadata: Box<Account<'info, MetadataAccount>>,

    #[account(
        seeds = [
            //also predefined by the metadata program that we need  to past these seeds
            b"metadata",
            metadata_program.key().as_ref(),
            maker_mint.key().as_ref(),
            b"edition",
        ],
        seeds::program = metadata_program.key(), //not owned by our program but by the metadata program
        bump
    )]
    pub master_edition: Box<Account<'info, MasterEditionAccount>>,

    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> List<'info> {
    pub fn create_listing(&mut self, price: u64, bumps: &ListBumps) -> Result<()> {
        self.listing.set_inner(Listing {
            maker: self.maker.key(),
            mint: self.maker_mint.key(),
            price,
            bump: bumps.listing,
        });

        Ok(())
    }
    pub fn deposit_nft(&mut self) -> Result<()> {
        let accounts = TransferChecked {
            from: self.maker_ata.to_account_info(),
            mint: self.maker_mint.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), accounts);
        transfer_checked(cpi_ctx, 1, self.maker_mint.decimals)?;
        Ok(())
    }
}

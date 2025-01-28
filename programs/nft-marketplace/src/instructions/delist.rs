use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    close_account,
    transfer_checked,
    CloseAccount,
    Mint,
    TokenAccount,
    TokenInterface,
    TransferChecked,
};

use crate::state::{ Listing, Marketplace };

#[derive(Accounts)]
#[instruction(name: String)]
pub struct Delist<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(seeds = [b"marketplace", name.as_str().as_bytes()], bump = marketplace.bump)]
    pub marketplace: Box<Account<'info, Marketplace>>,

    pub maker_mint: Box<InterfaceAccount<'info, Mint>>,
    pub collection_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,                                  //maybe init if needed cause the account could be closed already
        associated_token::mint = maker_mint,
        associated_token::authority = maker,
    )]
    pub maker_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        close = maker,
        seeds = [marketplace.key().as_ref(), maker_mint.key().as_ref()],
        bump
    )]
    pub listing: Box<Account<'info, Listing>>,

    #[account(
        mut,
        associated_token::authority = listing,
        associated_token::mint = maker_mint 
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Delist<'info> {
    pub fn withdraw_nft(&mut self) -> Result<()> {
        let seeds = &[
            self.marketplace.to_account_info().key.as_ref(),
            self.maker_mint.to_account_info().key.as_ref(),
            &[self.listing.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.maker_mint.to_account_info(),
            to: self.maker_ata.to_account_info(),
            authority: self.listing.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds
        );
        transfer_checked(cpi_ctx, 1, self.maker_mint.decimals)?;
        Ok(())
    }

    pub fn close_listing(&mut self) -> Result<()> {
        let accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.listing.to_account_info(),
        };

        let seeds = &[
            self.marketplace.to_account_info().key.as_ref(),
            self.listing.mint.as_ref(),
            &[self.listing.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds
        );

        close_account(ctx)?;
        Ok(())
    }
}

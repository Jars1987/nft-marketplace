use anchor_lang::{ prelude::*, system_program::{ transfer, Transfer } };
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account,
        transfer_checked,
        CloseAccount,
        Mint,
        TokenAccount,
        TokenInterface,
        TransferChecked,
        MintTo,
        mint_to,
    },
};

use crate::{ errors::MarketplaceError, state::{ Listing, Marketplace } };

#[derive(Accounts)]
pub struct Purchase<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    pub maker: SystemAccount<'info>, //do I need to pass the maker? Can I get it from the listing?
    pub maker_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::authority = taker,
        associated_token::mint = maker_mint
    )]
    pub taker_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
      mut,
      close = maker,
      seeds = [marketplace.key().as_ref(), listing.mint.as_ref()],
      bump = listing.bump
  )]
    pub listing: Account<'info, Listing>,

    #[account(
      mut,
      associated_token::authority = listing,
      associated_token::mint = maker_mint,
  )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
    mut,
    seeds = [b"mint", marketplace.key().as_ref()],
    bump = marketplace.rewards_mint_bump,
    mint::decimals = 6,
    mint::authority = marketplace,
)]
    pub rewards: Box<InterfaceAccount<'info, Mint>>,

    #[account(seeds = [b"treasury", marketplace.key().as_ref()], bump = marketplace.treasury_bump)]
    pub treasury: SystemAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Purchase<'info> {
    pub fn reward_buyer(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let accounts = MintTo {
            mint: self.rewards.to_account_info(),
            authority: self.marketplace.to_account_info(),
            to: self.taker.to_account_info(),
        };

        let seeds = &[b"marketplace", self.marketplace.name.as_bytes(), &[self.marketplace.bump]];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(cpi_program, accounts, signer_seeds);

        mint_to(ctx, 1);

        Ok(())
    }

    pub fn send_sol(&mut self) -> Result<()> {
        let feeAmmount = self.listing.price
            .checked_mul(self.marketplace.fee as u64) //multiply by the fee
            .and_then(|x| x.checked_div(10000)) //divide by 10000 because the fee is a percentage
            .unwrap();

        let totalAmmount = self.listing.price + feeAmmount;

        let accounts = Transfer {
            from: self.taker.to_account_info(),
            to: self.maker.to_account_info(), //it's a pubkey
        };

        let feeAccounts = Transfer {
            from: self.taker.to_account_info(),
            to: self.treasury.to_account_info(),
        };

        let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        let feeCtx = CpiContext::new(self.system_program.to_account_info(), feeAccounts);

        //Buyer pays the fees:
        transfer(ctx, self.listing.price)?;
        transfer(feeCtx, feeAmmount)?;
        Ok(())

        /* 

      //Seller pays the fees: 
      transfer(ctx, self.listing.price - feeAmmount))?;  //Taker to Maker --> the seller receives the price minus the fee
      transfer(feectx, feeAmmount))?;                   //Taker to Treasury --> the treasury receives the fee

    */
    }
    pub fn transfer_nft(&mut self) -> Result<()> {
        let accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.maker_mint.to_account_info(),
            to: self.taker_ata.to_account_info(),
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

        transfer_checked(ctx, 1, 0)?;
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

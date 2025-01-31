use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;

pub use instructions::*;
pub use state::*;

declare_id!("Dizu1VwarecyTVL8PAvHWmDBysRDy2Ns5mY1bMmhZ6VX");

#[program]
pub mod nft_marketplace {
    use super::*;

    pub fn init_marketplace(ctx: Context<Initialize>, name: String, fees: u16) -> Result<()> {
        ctx.accounts.init(name, fees, &ctx.bumps)?;
        Ok(())
    }

    pub fn list(ctx: Context<List>, price: u64) -> Result<()> {
        ctx.accounts.create_listing(price, &ctx.bumps)?;
        ctx.accounts.deposit_nft()?;
        Ok(())
    }

    pub fn delist(ctx: Context<Delist>) -> Result<()> {
        ctx.accounts.withdraw_nft()?;
        ctx.accounts.close_listing()?;
        Ok(())
    }

    pub fn purchase(ctx: Context<Purchase>) -> Result<()> {
        ctx.accounts.send_sol()?;
        ctx.accounts.transfer_nft()?;
        ctx.accounts.close_listing()?;
        ctx.accounts.reward_buyer()?;
        Ok(())
    }
}

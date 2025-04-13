#![allow(clippy::result_large_err)] // лише на початку !
use anchor_lang::prelude::*;

pub mod instructions;
use instructions::*;
// declare_id!("CHGNy7GAQZTRVwXcD3WDwhTRrcuxhFdJSe5H1hbgyXiw");
// declare_id!("CHGNy7GAQZTRVwXcD3WDwhTRrcuxhFdJSe5H1hbgyXiw");

declare_id!("HbaV26KFPVavib8iMm6PqybTXiMchxYUXmGkRss9AF24");
// Specifies the module containing the program’s instruction logic
#[program]
pub mod mint_spl_token {
    use super::*;

    pub fn create_token(
        ctx: Context<CreateToken>,
        token_name: String,
        token_symbol: String,
        token_uri: String,
    ) -> Result<()> {
        create::create_token(
            ctx,
            token_name,
            token_symbol,
            token_uri,
        )
    }
    
    pub fn mint_token(
        ctx: Context<MintToken>,
        amount: u64,
    ) -> Result<()> {
        mint::mint_token(ctx, amount)
    }
    
    pub fn exchange_nft_for_tokens(
        ctx: Context<ExchangeNFTForTokens>,
        amount: u64,
    ) -> Result<()> {
        exchange_nft_for_tokens::exchange_nft_for_tokens(ctx, amount)
    }

    pub fn investor_vesting_tokens(
        ctx: Context<InvestorVestingTokens>,
        total_amount: u64,
        vesting_duration: u64, // in seconds
        tge_percentage: u16, // percentage in basic points! 1 % - 100 bps
    ) -> Result<()> {
        vesting::investor_vesting_tokens(ctx, total_amount, vesting_duration, tge_percentage)
    }

    pub fn claim_vested_tokens(
        ctx: Context<ClaimVestedTokens>,
    ) -> Result<()> {
        vesting::claim_vested_tokens(ctx)
    }
}
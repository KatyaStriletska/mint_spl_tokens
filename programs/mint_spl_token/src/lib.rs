#![allow(clippy::result_large_err)] // лише на початку !
use anchor_lang::prelude::*;

pub mod instructions;
use instructions::*;
// declare_id!("CHGNy7GAQZTRVwXcD3WDwhTRrcuxhFdJSe5H1hbgyXiw");
// declare_id!("CHGNy7GAQZTRVwXcD3WDwhTRrcuxhFdJSe5H1hbgyXiw");

declare_id!("FSr2mU8qEKqwVYcFGo1H2JRSSvQ4Ln1v477Hq2kgVfN");
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
}
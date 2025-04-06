use anchor_lang::prelude::*;

declare_id!("CHGNy7GAQZTRVwXcD3WDwhTRrcuxhFdJSe5H1hbgyXiw");

#[program]
pub mod mint_spl_token {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

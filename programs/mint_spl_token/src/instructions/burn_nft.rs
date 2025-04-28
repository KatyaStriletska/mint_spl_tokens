use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{close_account, CloseAccount, TokenAccount, Mint, Token, burn, Burn}
    },
};
// Test instruction for burn and close NFT account
pub fn burn_nft(
    ctx:Context<BurnNft>,
) -> Result<()>{
 
    // 1. Burn NFT 
    msg!("Burning NFT...");

    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn{
                mint: ctx.accounts.nft_mint.to_account_info(),
                from: ctx.accounts.nft_token_account.to_account_info(),
                authority: ctx.accounts.investor.to_account_info(),
            }, 
        ),  
        1, // NFT amount 
    )?;
    msg!("NFT burned successfully.");

    // 2.close nft token account
    msg!("Closing NFT token account...");
    close_account(
        CpiContext::new (
            ctx.accounts.token_program.to_account_info(),
            CloseAccount{
                // pub account: AccountInfo<'info>,
                account: ctx.accounts.nft_token_account.to_account_info(),
                // pub destination: AccountInfo<'info>,
                destination: ctx.accounts.investor.to_account_info(),
                // pub authority: AccountInfo<'info>,
                authority: ctx.accounts.investor.to_account_info(),
            }
        )
    )?;
    msg!("NFT token account closed successfully.");
   
    Ok(())
}


#[derive(Accounts)]
pub struct BurnNft<'info>{
    #[account(mut)]
    pub investor: Signer<'info>,
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,
    // #[account(mut, has_one = investor)]
    #[account( 
        mut, 
        associated_token::mint = nft_mint, 
        associated_token::authority = investor,
        constraint = nft_token_account.amount == 1 
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    // pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program:Program<'info, System>,
}

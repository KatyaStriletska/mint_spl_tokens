use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{mint_to, MintTo, TokenAccount, Mint, Token, burn, Burn}
    },
};

pub fn exchange_nft_for_tokens(
    ctx:Context<ExchangeNFTForTokens>,
    amount: u64,
) -> Result<()>{
    msg!("Minting tokens to associated token account...");
    msg!("Mint: {}", &ctx.accounts.fungible_mint.key());
    msg!(
        "Token Address: {}",
        &ctx.accounts.associated_token_account.key()
    );
    
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

    // 2. Mint fungible tokens 
    msg!("Minting fungible tokens...");
    mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo{
                mint: ctx.accounts.fungible_mint.to_account_info(),
                to: ctx.accounts.associated_token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
        ),
        amount * 10u64.pow(ctx.accounts.fungible_mint.decimals as u32), // Mint tokens, adjust for decimals
    )?;
    msg!("Token minted successfully.");

    Ok(())
}


#[derive(Accounts)]
pub struct ExchangeNFTForTokens<'info>{
    #[account(mut)]
    pub mint_authority: Signer<'info>,
    #[account(mut)]
    pub fungible_mint:Account<'info, Mint>,
    #[account(mut)]
    pub investor: Signer<'info>,
    // це токен-акаунт отримувача для конкретного типу токена (mint_account)
    // mut
    #[account(
        init_if_needed,
        payer = investor,
        associated_token::mint = fungible_mint,
        associated_token::authority = investor,
    )]    
    pub associated_token_account: Account<'info, TokenAccount>,

   
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,
    // #[account(mut, has_one = investor)]
    #[account( 
        mut, // mut, бо зменшується amount і акаунт буде закрито
        associated_token::mint = nft_mint, // Перевірка, що ATA для правильного мінта NFT
        associated_token::authority = investor, // Перевірка, що власник ATA - це інвестор, що підписав
        constraint = nft_token_account.amount == 1 //@ ErrorCode::NftAccountEmptyOrWrongAmount
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program:Program<'info, System>,
}
#[error_code]
pub enum ErrorCode {
    #[msg("NFT token account is empty or does not hold exactly one NFT.")]
    NftAccountEmptyOrWrongAmount,
    #[msg("Amount calculation overflowed.")]
    AmountOverflow,
}
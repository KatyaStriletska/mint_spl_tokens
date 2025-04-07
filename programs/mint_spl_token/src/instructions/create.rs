use {
    anchor_lang::prelude::*,
    anchor_spl::{
        metadata::{
            create_metadata_accounts_v3, 
            mpl_token_metadata::types::DataV2, 
            CreateMetadataAccountsV3, 
            Metadata,
        },
        token::{Mint, Token}
    },
};

pub fn create_token(
    ctx: Context<CreateToken>,
    token_name: String,
    token_symbol: String, 
    token_uri: String, 
) -> Result<()>{
    msg!("Starting token creation...");

    msg!("Starting metadata creation...");
    create_metadata_accounts_v3(
        CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3{
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.mint_account.to_account_info(),
                mint_authority: ctx.accounts.payer.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            }, 
        ),
        DataV2{
            name: token_name, 
            symbol: token_symbol, 
            uri: token_uri,
            seller_fee_basis_points: 0, // роялті з продаж, автоматично сплачується творцям коли продається токен
            // optional
            creators: None, 
            collection: None,
            uses: None, // не передбачено механізму обмеженого використання 
        }, 
        true,
        true,
        None,
    )?;
    msg!("Metadata created successfully!");
    msg!("Token created successfully!");

    Ok(())
}

#[derive(Accounts)]
// info - lifetime, щоб щоб Rust знав, як довго живуть посилання на акаунти.
pub struct CreateToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    #[account(
        init, 
        payer = payer,
        mint::decimals = 9,
        mint::authority = payer.key(),
        mint::freeze_authority = payer.key(),
    )]
    pub mint_account: Account<'info, Mint>,
    /// CHECK: Metaplex will check this account: 
    #[account(
        mut, 
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint_account.key().as_ref()],
        bump, 
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata_account: UncheckedAccount<'info>,
}
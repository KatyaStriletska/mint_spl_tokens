use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self,
            mint_to,
            MintTo, 
            TokenAccount, 
            Mint, 
            Token, 
            burn, 
            Burn, 
            transfer, 
            Transfer, 
            close_account, 
            CloseAccount,
            SetAuthority
        }
    },
    spl_token::instruction::AuthorityType,
};

pub fn start_vesting_from_vault(
    ctx:Context<StartVestingFromVault>,
    total_amount: u64,
    vesting_duration: u64, // in seconds
    tge_percentage: u16, // percentage in basic points! 1 % - 100 bps
) -> Result<()>{
    msg!("Starting vesting from MAIN vault...");

    // 0. Calculate TGE amount
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;
    
    // це перевід у нативний формат який збергається на блокчейні
    let decimals = ctx.accounts.fungible_mint.decimals;
    let total_amount_native = total_amount * 10u64.pow(decimals as u32);

    let tge_amount_native = u128::from(total_amount_native) // щоб уникнути переповнення під час множення
    // юзаємо інто для того щоб конвертувати тип значення tge_percentage в тип, який вказаний після into()
        .checked_mul(tge_percentage.into()) 
        .unwrap_or(0)
        .checked_div(10000) 
        .unwrap_or(0) as u64;

    let vesting_amount_native = total_amount_native
        .checked_sub(tge_amount_native)
        .ok_or(ErrorCode::AmountCalculationError)?; // TODO!
    
    msg!("Total Allocation (native): {}", total_amount_native);
    msg!("TGE Amount (native): {}", tge_amount_native);
    msg!("Vesting Amount (native): {}", vesting_amount_native);

    if ctx.accounts.main_token_vault.amount < total_amount_native {
        return Err(ErrorCode::InsufficientFundsInMainVault.into());
    }
    let mint_key = ctx.accounts.fungible_mint.key();
    let (_pda, authority_bump) = Pubkey::find_program_address(
         &[b"main_vault_authority", mint_key.as_ref()], 
         ctx.program_id 
    );
    let main_authority_seeds = &[
        b"main_vault_authority".as_ref(),
        mint_key.as_ref(),
        &[authority_bump], 
    ];
    let signer_seeds = &[&main_authority_seeds[..]]; 

  // 3. Mint fungible tokens 
    msg!("Minting fungible tokens...");
    if tge_amount_native > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer { 
                    from: ctx.accounts.main_token_vault.to_account_info(),
                    to: ctx.accounts.associated_token_account.to_account_info(),
                    authority: ctx.accounts.main_vault_authority.to_account_info(), // PDA авторизує
                },
                signer_seeds, 
            ),
            tge_amount_native,
        )?;
        msg!("TGE tokens transferred.");
    }

    if vesting_amount_native > 0 {
        msg!("Transferring vesting tokens to individual vesting vault...");
         token::transfer( 
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                 token::Transfer {
                    from: ctx.accounts.main_token_vault.to_account_info(),
                    to: ctx.accounts.vesting_vault.to_account_info(), 
                    authority: ctx.accounts.main_vault_authority.to_account_info(), 
                },
                signer_seeds,
            ),
            vesting_amount_native,
        )?;
        msg!("Vesting tokens transferred to vault.");
    }

    // 4. Create vesting account
    let vesting_account = &mut ctx.accounts.vesting_account;
    vesting_account.beneficiary = ctx.accounts.investor.key();
    vesting_account.mint = ctx.accounts.fungible_mint.key();
    vesting_account.token_account = ctx.accounts.vesting_vault.key();
    vesting_account.total_amount = vesting_amount_native;
    vesting_account.start_time = current_timestamp;
    vesting_account.duration = vesting_duration;
    vesting_account.released_amount = 0;
    vesting_account.tge_amount = tge_amount_native;
    msg!("Vesting account initialized for beneficiary: {}", ctx.accounts.investor.key());
    msg!("Vesting starts at: {}, duration: {} seconds", current_timestamp, vesting_duration);

    Ok(())
}
pub fn claim_vested_tokens(ctx: Context<ClaimVestedTokens>) -> Result<()>{
    let vesting_account = &mut ctx.accounts.vesting_account;

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    // тут можна заюзати require_keys_eq!(...) макрос для перевірки ключів на рівність 
    if vesting_account.beneficiary != *ctx.accounts.beneficiary.key {
        return Err(ErrorCode::BeneficiaryNotFound.into());
    }
    let elapsed_time = current_timestamp
        .checked_sub(vesting_account.start_time)
        .unwrap_or(0);

    let vested_amount = if elapsed_time <= 0 || vesting_account.duration == 0 {
        0
    } else if elapsed_time >= vesting_account.duration as i64 {
        vesting_account.total_amount
    } else {
        u128::from(vesting_account.total_amount)
            .checked_mul(elapsed_time as u128).unwrap_or(0)
            .checked_div(vesting_account.duration as u128).unwrap_or(0)
            as u64
    };

    let claimable_amount = vested_amount
        .checked_sub(vesting_account.released_amount)
        .ok_or(ErrorCode::AmountCalculationError)?; 

    if claimable_amount == 0 {
        msg!("Nothing to claim at this time.");
        return Err(ErrorCode::NothingToClaim.into());
    }

    msg!(
        "Elapsed time: {}s. Total Vested: {}. Released: {}. Claimable: {}",
        elapsed_time,
        vested_amount,
        vesting_account.released_amount,
        claimable_amount
    );

    let seeds = &[
        b"vault_authority",
        vesting_account.beneficiary.as_ref(),
        vesting_account.mint.as_ref(),
        &[ctx.bumps.vault_authority], 
    ];
    let signer_seeds = &[&seeds[..]];

    let transfer_instruction = Transfer {
        from: ctx.accounts.vesting_vault.to_account_info(),
        to: ctx.accounts.beneficiary_token_account.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
        signer_seeds
    );

    transfer(cpi_ctx, claimable_amount)?;
    vesting_account.released_amount = vesting_account.released_amount
    .checked_add(claimable_amount)
    .ok_or(ErrorCode::AmountCalculationError)?;

    msg!("Claim successful. New released amount: {}", vesting_account.released_amount);

    Ok(())
}

#[derive(Accounts)]
pub struct ClaimVestedTokens<'info>{
    #[account(mut)]
    pub beneficiary: Signer<'info>,

    #[account(
        mut,
        seeds = [
            b"vesting",
            beneficiary.key().as_ref(),
            vesting_account.mint.as_ref() 
        ],
        bump,
        // constraint = vesting_account.beneficiary == beneficiary.key() // Додаткова перевірка
    )]
    pub vesting_account: Account<'info, VestingAccount>,
    // PDA, який буде authority для vesting_vault
    /// CHECK: Це просто PDA, не треба читати його дані. Використовується як signer.
    #[account(
        seeds = [
            b"vault_authority",
            beneficiary.key().as_ref(),
            vesting_account.mint.as_ref()
        ],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    // Token Account (сховище), де зберігаються заблоковані токени (PDA)
    #[account(
        mut,
        // token::mint = fungible_mint,
        token::authority = vault_authority, 
        seeds = [
            b"vault", 
            beneficiary.key().as_ref(),
            vesting_account.mint.as_ref()
        ],
        bump,
        // constraint = vesting_vault.key() == vesting_account.vault @ ErrorCode::VaultMismatch // Перевірка відповідності
    )]
    pub vesting_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = vesting_account.mint, 
        associated_token::authority = beneficiary, 
    )]
    pub beneficiary_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>, 
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}


#[derive(Accounts)]
pub struct StartVestingFromVault<'info>{
    pub fungible_mint:Account<'info, Mint>,
    #[account(mut)]
    pub investor: Signer<'info>,
    #[account(
        init_if_needed,
        payer = investor,
        associated_token::mint = fungible_mint,
        associated_token::authority = investor,
    )]    
    pub associated_token_account: Account<'info, TokenAccount>,
    #[account(
        mut, 
        seeds = [
            b"main_vault", 
            fungible_mint.key().as_ref()
        ],
        bump 
    )]
    pub main_token_vault: Account<'info, TokenAccount>,
    /// CHECK: PDA, що авторизує перекази з main_token_vault.
    #[account(
        seeds = [
            b"main_vault_authority",
            fungible_mint.key().as_ref()
        ],
        bump 
    )]
    pub main_vault_authority: AccountInfo<'info>, 

    #[account(
        init_if_needed,
        payer = investor,
        space = VestingAccount::LEN + 8,
        seeds = [
            b"vesting",
            investor.key().as_ref(), 
            fungible_mint.key().as_ref()],
        bump
    )]
    pub vesting_account: Account<'info, VestingAccount>,
    // PDA, який буде authority для vesting_vault
    /// CHECK: Це просто PDA, не треба читати його дані. Використовується як signer.
    #[account(
        seeds = [
            b"vault_authority",
            investor.key().as_ref(),
            fungible_mint.key().as_ref()
        ],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = investor, 
        token::mint = fungible_mint,
        token::authority = vault_authority, 
        seeds = [
            b"vault",
            investor.key().as_ref(),
            fungible_mint.key().as_ref()
        ],
        bump
    )]
    pub vesting_vault: Account<'info, TokenAccount>,

    // System programs
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program:Program<'info, System>,
    pub clock: Sysvar<'info, Clock>, // чи треба? 

}

#[error_code]
pub enum ErrorCode {
    #[msg("NFT token account is empty or does not hold exactly one NFT.")]
    NftAccountEmptyOrWrongAmount,
    #[msg("Amount calculation overflowed.")]
    AmountOverflow,
    #[msg("Amount calculation error.")]
    AmountCalculationError,
    #[msg("Nothing to claim.")]
    NothingToClaim,
    #[msg("Beneficiary not found.")]    
    BeneficiaryNotFound,
    #[msg("Insufficient funds in main vault.")] 
    InsufficientFundsInMainVault,
    #[msg("Bump seed not found in context.")] 
    BumpSeedNotFound,

}

#[account]
#[derive(Default)]
pub struct VestingAccount {
    pub beneficiary: Pubkey,
    pub mint: Pubkey,
    pub token_account: Pubkey, 
    pub total_amount: u64,

    pub start_time: i64,
    pub duration: u64,
    pub released_amount: u64,
    pub tge_amount: u64, 

    //  pub cliff_duration: u64,  
    // pub period_length: u64,   
}
impl VestingAccount {
    // Приблизний розмір акаунту для розрахунку rent
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 /* + 8 + 8 */;
}

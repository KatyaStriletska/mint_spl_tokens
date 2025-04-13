use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{mint_to, MintTo, TokenAccount, Mint, Token, burn, Burn, transfer, Transfer}
    },
};

pub fn investor_vesting_tokens(
    ctx:Context<InvestorVestingTokens>,
    total_amount: u64,
    vesting_duration: u64, // in seconds
    tge_percentage: u16, // percentage in basic points! 1 % - 100 bps
) -> Result<()>{
    msg!("Minting tokens to associated token account...");
    msg!("Mint: {}", &ctx.accounts.fungible_mint.key());
    msg!(
        "Token Address: {}",
        &ctx.accounts.associated_token_account.key()
    );

    // 0. Calculate TGE amount
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;
    
    // це перевід у нативний формат який збергається на блокчейні
    let decimals = ctx.accounts.fungible_mint.decimals;
    let total_amount_native = total_amount * 10u64.pow(decimals as u32);

    let tge_amount_native = u128::from(total_amount_native) // щоб уникнути переповнення під час множення
    // юзаємо інто для того щоб конвертувати тип значення tge_percentage в тип, який вказаний після into()
        .checked_mul(tge_percentage.into()) // множить загальну нативну суму на вітсоток у bps
        .unwrap_or(0)
        .checked_div(10000) // 100% = 10000 basis points
        .unwrap_or(0) as u64;

    let vesting_amount_native = total_amount_native
        .checked_sub(tge_amount_native)
        .ok_or(ErrorCode::AmountCalculationError)?; // TODO!
    
    msg!("Total Allocation (native): {}", total_amount_native);
    msg!("TGE Amount (native): {}", tge_amount_native);
    msg!("Vesting Amount (native): {}", vesting_amount_native);


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
    // TODO: close_account(...)

    msg!("NFT burned successfully.");

    // 2. Mint fungible tokens 
    msg!("Minting fungible tokens...");
    if tge_amount_native > 0 {
        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo{
                    mint: ctx.accounts.fungible_mint.to_account_info(),
                    to: ctx.accounts.associated_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
            ),
            tge_amount_native,
        )?;
        msg!("TGE Token minted successfully.");
    } 
    if vesting_amount_native > 0 {
        msg!("Minting vesting tokens to vault...");
        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo{
                    mint: ctx.accounts.fungible_mint.to_account_info(),
                    to: ctx.accounts.vesting_vault.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
            ),
            vesting_amount_native,
        )?;
        msg!("Vesting Token minted successfully.");
    } else {
        msg!("No tokens to vest.");
    }

    // 3. Create vesting account
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
// бенефіціар викликає цю функцію, щоб отримати токени з вестінгу
pub fn claim_vested_tokens(ctx: Context<ClaimVestedTokens>) -> Result<()>{
    let vesting_account = &mut ctx.accounts.vesting_account;

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    // тут можна заюзати require_keys_eq!(...) макрос для перевірки ключів на рівність 
    if vesting_account.beneficiary != *ctx.accounts.beneficiary.key {
        return Err(ErrorCode::BeneficiaryNotFound.into());
    }
    // Розрахунок доступної суми для клейму (лінійний вестінг)
    let elapsed_time = current_timestamp
        .checked_sub(vesting_account.start_time)
        .unwrap_or(0);

    // Рекомендована структура розрахунку vested_amount
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
        &[ctx.bumps.vault_authority], // Bump seed для PDA-authority сховища
    ];
    let signer_seeds = &[&seeds[..]];

    // Transfer tokens to beneficiary's wallet
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
        // Можна додати close = beneficiary, щоб акаунт закрився після повного вестінгу,
        // але це потребує додаткової логіки в інструкції claim.
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
            b"vault", // Відрізняється від seed для vesting_account
            beneficiary.key().as_ref(),
            vesting_account.mint.as_ref()
        ],
        bump,
        // constraint = vesting_vault.key() == vesting_account.vault @ ErrorCode::VaultMismatch // Перевірка відповідності
    )]
    pub vesting_vault: Account<'info, TokenAccount>,
    // Асоційований токен-акаунт бенефіціара для отримання розблокованих токенів
    #[account(
        mut,
        associated_token::mint = vesting_account.mint, // Має бути той самий токен
        associated_token::authority = beneficiary, // Власник - той, хто клеймить
    )]
    pub beneficiary_token_account: Account<'info, TokenAccount>,


    // Системні програми
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>, // Може не знадобитись тут
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>, // Потрібен для отримання поточного часу
}


#[derive(Accounts)]
pub struct InvestorVestingTokens<'info>{
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

    // NFT 
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,
    // #[account(mut, has_one = investor)]
    #[account( 
        mut, // mut, бо зменшується amount і акаунт буде закрито
        associated_token::mint = nft_mint, // Перевірка, що ATA для правильного мінта NFT
        associated_token::authority = investor, // Перевірка, що власник ATA - це інвестор, що підписав
        // constraint = nft_token_account.amount == 1 //@ ErrorCode::NftAccountEmptyOrWrongAmount
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    // Vesting
    // Акаунт для зберігання даних вестінгу (PDA)
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
    // Token Account (сховище), де зберігаються заблоковані токени (PDA)
    #[account(
        init_if_needed,
        payer = investor, // Інвестор платить за створення сховища
        token::mint = fungible_mint,
        token::authority = vault_authority, 
        seeds = [
            b"vault", // Відрізняється від seed для vesting_account
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

}

#[account]
#[derive(Default)]
pub struct VestingAccount {
    pub beneficiary: Pubkey,
    pub mint: Pubkey,
    pub token_account: Pubkey, // vault, тут зберігаються заблоковані токени (PDA)
    pub total_amount: u64,

    pub start_time: i64,
    pub duration: u64,
    pub released_amount: u64,
    pub tge_amount: u64, 

    //  pub cliff_duration: u64,  // Тривалість кліффу в секундах (період, коли нічого не розблоковується)
    // pub period_length: u64,   // Довжина періоду розблокування (напр. місяць) - для ступінчастого вестінгу
}
impl VestingAccount {
    // Приблизний розмір акаунту для розрахунку rent
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 /* + 8 + 8 */;
}

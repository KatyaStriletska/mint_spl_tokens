import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MintSplToken } from "../target/types/mint_spl_token";
import { PublicKey } from "@solana/web3.js";
import { getAssociatedTokenAddressSync } from "@solana/spl-token"; 
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token"; 

describe("INVESTOR VESTING AND BURN NFT", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.mintSplToken as Program<MintSplToken>;

    const payer = provider.wallet; 

    const total_amount = new anchor.BN(100); // amount of investments
    const vesting_duration = new anchor.BN(30 * 60); // 1 hour
    const tge_percentage = 1000; // 10% of total amount (1000 bps)

    const fungibleMintAddress = new PublicKey("DNMVCRJKfHe4yz5tcgteSn6e75WHXUtoZZxKjcii2uze");

    const [vestingAccountPda, vestingAccountBump] =
        PublicKey.findProgramAddressSync(
            [
                Buffer.from("vesting"),
                payer.publicKey.toBuffer(),
                fungibleMintAddress.toBuffer(),
            ],
            program.programId
        );

    const [vaultAuthorityPda, vaultAuthorityBump] =
        PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault_authority"),
                payer.publicKey.toBuffer(),
                fungibleMintAddress.toBuffer(),
            ],
            program.programId
        );

    const [vestingVaultPda, vestingVaultBump] =
        PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                payer.publicKey.toBuffer(),
                fungibleMintAddress.toBuffer(),
            ],
            program.programId
        );

    const associatedTokenAccountAddress = getAssociatedTokenAddressSync(
        fungibleMintAddress,
        payer.publicKey 
    );

    it("Should  mint some % of tokens", async () => {
        try {
            const tx = await program.methods
                .investorVestingTokens(
                    total_amount,
                    vesting_duration,
                    tge_percentage,
                )
                .accountsStrict({
                    mintAuthority: payer.publicKey,
                    fungibleMint: fungibleMintAddress,
                    investor: payer.publicKey,
                    associatedTokenAccount: associatedTokenAccountAddress, 
                    vestingAccount: vestingAccountPda, 
                    vaultAuthority: vaultAuthorityPda, 
                    vestingVault: vestingVaultPda, 
                    tokenProgram: TOKEN_PROGRAM_ID,              
                    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID, 
                    systemProgram: anchor.web3.SystemProgram.programId, 
                })
                .signers([payer.payer]) 
                .rpc();

            console.log("Initialization Transaction Signature:", tx);
            console.log("\n\x1b[32m%s\x1b[0m", "=== Initialization Successful! ===");
            console.log(`\n>>> Запам'ятайте адресу Vesting Account PDA: ${vestingAccountPda.toBase58()}`);
            console.log(`>>> Вестінг триватиме 1 годину.`);
            console.log(`>>> Починайте запускати тест клейму (investorVesting.claim.ts) через кілька хвилин.`);
        } catch (error) {
            console.error("Error during initialization:", error); 
            if (error instanceof anchor.AnchorError) {
                console.error("\nProgram Logs:", error.logs);
            }
            throw error;
        }
    });

    // Тут має бути ваш другий тест "Should claim tokens after vesting period"
    // Переконайтесь, що він також передає всі необхідні акаунти для ClaimVestedTokens
    // Наприклад:
    // it("Should claim tokens after vesting period", async () => {
    //    // Затримка для вестінгу
    //    await new Promise(resolve => setTimeout(resolve, 2000)); // Зачекайте трохи часу (напр. 2 сек)
    //
    //    const beneficiaryTokenAccountAddress = getAssociatedTokenAddressSync(
    //        fungibleMintAddress,
    //        payer.publicKey
    //    );
    //
    //    try {
    //        const tx = await program.methods
    //            .claimVestedTokens()
    //            .accounts({
    //                beneficiary: payer.publicKey,
    //                vestingAccount: vestingAccountPda,
    //                vaultAuthority: vaultAuthorityPda,
    //                vestingVault: vestingVaultPda,
    //                beneficiaryTokenAccount: beneficiaryTokenAccountAddress,
    //                tokenProgram: TOKEN_PROGRAM_ID,
    //                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    //                systemProgram: anchor.web3.SystemProgram.programId,
    //                // clock: anchor.web3.SYSVAR_CLOCK_PUBKEY, // Clock передається автоматично Anchor'ом
    //            })
    //            .rpc({ commitment: "confirmed" });
    //        console.log("Claim Transaction Signature:", tx);
    //        // Перевірки балансу і т.д.
    //    } catch (error) {
    //         console.error("Error during claim:", error);
    //         if (error instanceof anchor.AnchorError) {
    //             console.error("\nProgram Logs:", error.logs);
    //         }
    //         throw error;
    //     }
    // });
});
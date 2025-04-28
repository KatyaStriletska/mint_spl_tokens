import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MintSplToken } from "../target/types/mint_spl_token";
import {
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY, 
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token"; 

describe("INVESTOR CLAIM VESTED TOKENS", () => { 
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.MintSplToken as Program<MintSplToken>;
  const connection = provider.connection;

  const payer = provider.wallet;
  const fungibleMintAddress = new PublicKey("DNMVCRJKfHe4yz5tcgteSn6e75WHXUtoZZxKjcii2uze");
  
  const [vestingAccountPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("vesting"),
      payer.publicKey.toBuffer(),
      fungibleMintAddress.toBuffer(),
    ],
    program.programId
  );

  const [vaultAuthorityPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("vault_authority"),
      payer.publicKey.toBuffer(),
      fungibleMintAddress.toBuffer(),
    ],
    program.programId
  );

  const [vestingVaultPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("vault"), 
      payer.publicKey.toBuffer(),
      fungibleMintAddress.toBuffer(),
    ],
    program.programId
  );

  const beneficiaryTokenAccountAddress = getAssociatedTokenAddressSync(
    fungibleMintAddress,
    payer.publicKey 
  );

  it("Should claim tokens after vesting period", async () => {
  
    try {
      console.log(`Attempting to claim tokens for beneficiary: ${payer.publicKey.toBase58()}`);
      console.log(`Using Vesting Account PDA: ${vestingAccountPda.toBase58()}`);
      console.log(`Using Vesting Vault PDA: ${vestingVaultPda.toBase58()}`);
      console.log(`Target Beneficiary ATA: ${beneficiaryTokenAccountAddress.toBase58()}`);

      let initialBalance = new anchor.BN(0);
      try {
        const tokenAccountInfo = await connection.getTokenAccountBalance(beneficiaryTokenAccountAddress);
        initialBalance = new anchor.BN(tokenAccountInfo.value.amount);
        console.log(`Initial balance in beneficiary ATA: ${tokenAccountInfo.value.uiAmountString}`);
      } catch (e) {
         console.log("Beneficiary ATA might not exist yet or has 0 balance.");
      }


      const tx = await program.methods
        .claimVestedTokens()
        .accountsStrict({ 
          beneficiary: payer.publicKey,
          vestingAccount: vestingAccountPda,
          vaultAuthority: vaultAuthorityPda, 
          vestingVault: vestingVaultPda,
          beneficiaryTokenAccount: beneficiaryTokenAccountAddress,
          tokenProgram: TOKEN_PROGRAM_ID,
          clock: SYSVAR_CLOCK_PUBKEY, 
           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
           systemProgram: SystemProgram.programId,
        })
        .rpc({ commitment: "confirmed" });

      console.log("Claim Transaction Signature:", tx);
      console.log("\n\x1b[32m%s\x1b[0m", "=== Claim Successful! ===");

      await new Promise(resolve => setTimeout(resolve, 1000)); 
      const finalTokenAccountInfo = await connection.getTokenAccountBalance(beneficiaryTokenAccountAddress);
      const finalBalance = new anchor.BN(finalTokenAccountInfo.value.amount);
      console.log(`Final balance in beneficiary ATA: ${finalTokenAccountInfo.value.uiAmountString}`);
      const claimedAmount = finalBalance.sub(initialBalance);
      console.log(`Claimed amount: ${claimedAmount.toString()} raw units`);

    } catch (error) {
      console.error("Error during claimVestedTokens:", error); 
      if (error instanceof anchor.AnchorError) {
        console.error("\nProgram Logs:", error.logs);
      }
      throw error;
    }
  });
});
//6ZuuyawMxPJLE4qACiKUBPwjU9xj78avTJ9gkJPbnKdy
// Initialization Transaction Signature: 3gfjeirLyugJxNSzFtkmSLpZvG6XiRLPAJWP5zggxXVkyvCWnqyN2xm7GDe7fkQTfue4PBiJdsajjxevrMLi8UYu
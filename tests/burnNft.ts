import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MintSplToken } from "../target/types/mint_spl_token";
import {
  PublicKey,
} from "@solana/web3.js";

describe("BURN NFT AND CLOSE ACCOUNT", () => { 
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.MintSplToken as Program<MintSplToken>;

  const payer = provider.wallet; 
  const nftMint = new PublicKey("FEdhugZk5HXLnJi6USgcAcve1rPmnbrHhRpyHSe96fhK"); 
 
  it("Should burn and close nft account with id = My Anchor NFT #574201", async () => {
   
    try {
      const tx = await program.methods
        .burnNft()
        .accounts({
          investor: payer.publicKey,
          nftMint: nftMint,
        })
        .rpc({ commitment: "confirmed" });

      console.log("Claim Transaction Signature:", tx);
      console.log("\n\x1b[32m%s\x1b[0m", "=== Claim Successful! ===");
    } catch (error) {
      console.error("Error during claimVestedTokens:", error);
      if (error instanceof anchor.AnchorError) {
        console.error("\nProgram Logs:", error.logs);
      }
      throw error;
    }
  });
});
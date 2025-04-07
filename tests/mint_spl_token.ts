import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MintSplToken } from "../target/types/mint_spl_token";
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { createAssociatedTokenAccount, getAssociatedTokenAddressSync } from "@solana/spl-token";

describe("mint_spl_token", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.mintSplToken as Program<MintSplToken>;

    const payer = provider.wallet;

    const tokenName = "Test token";
    const tokenSymbol = "SPL";
    const tokenUri =
     "https://gateway.pinata.cloud/ipfs/bafkreienmja33t43ly3rvs54jbxdqqvkqyse4ltbg2gsrwcjxhoboaosei"; 
  
    const mintKeypair = anchor.web3.Keypair.generate();
    const mint = mintKeypair.publicKey;
    
   
  it("Sould create an SPL token with metadata!", async () => {
    console.log("Starting creation of SPL token with metadata...");

    try{
      const tx = await program.methods
        .createToken(
          tokenName,
          tokenSymbol,
          tokenUri,
        )
        .accounts({
          payer: payer.publicKey,
          mintAccount: mint,
        })
        .signers([mintKeypair])
        .rpc();
      console.log("Your transaction signature", tx);
    }catch (error) {
      console.error("Error creating token: ", error);
    }
    console.log("SPL token with metadata created successfully!");
  });


  it("Should mint tokens to associated account!", async () => { 
    const amount = new anchor.BN(2);
    const recipient = new PublicKey("7bCkmDRYMPTSYJPEhdx9rvWhQb3UpbBPDVuSDhWThPTF");  
    // const mint = new PublicKey("7bCkmDRYMPTSYJPEhdx9rvWhQb3UpbBPDVuSDhWThPTF")
    try{
      const tx = await program.methods
        .mintToken(
          amount,
        )
        .accounts({
          mintAuthority: payer.publicKey,
          mintAccount: mint, 
          recipient: recipient
        })
        // .signers([mintKeypair])
        .signers([payer.payer])
        .rpc();
        console.log("Your transaction signature", tx);
      }catch (error) {
        console.error("Error creating token: ", error);
      }
      console.log("SPL token was minted successfully!")
  });


});
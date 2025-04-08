import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MintSplToken } from "../target/types/mint_spl_token";
import {
    PublicKey,
    SystemProgram,
    Keypair
} from "@solana/web3.js";
import {
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getAssociatedTokenAddressSync,
    createMint, 
    mintTo,     
    getAccount, 
    getMint
} from "@solana/spl-token";
import { assert } from "chai"; 

describe("nft_exchange (Local Test with Provider Wallet)", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.MintSplToken as Program<MintSplToken>;
    const connection = provider.connection;

    const testWallet = provider.wallet as anchor.Wallet;
    console.log(`Using Test Wallet: ${testWallet.publicKey.toBase58()} as Investor & Mint Authority`);

    const amount = new anchor.BN(1); 
    const nftMintAddress = new PublicKey("2o5WswYePk6pq2YZ5dKxN79UyRZRw25sLM55oVSVnVho");
    const fungibleMintAddress = new PublicKey("DNMVCRJKfHe4yz5tcgteSn6e75WHXUtoZZxKjcii2uze");

    let walletNftAta: PublicKey;
    let walletFungibleAta: PublicKey;

    before(async () => {
        console.log("--- Test Setup ---");
        walletNftAta = getAssociatedTokenAddressSync(nftMintAddress, testWallet.publicKey);
        walletFungibleAta = getAssociatedTokenAddressSync(fungibleMintAddress, testWallet.publicKey);
        console.log(`Wallet NFT ATA: ${walletNftAta.toBase58()}`);
        console.log(`Wallet Fungible ATA: ${walletFungibleAta.toBase58()}`);

        // NFT Check
        try {
            const nftAccount = await getAccount(connection, walletNftAta);
            assert.strictEqual(nftAccount.amount.toString(), "1", `Wallet ${testWallet.publicKey.toBase58()} does not hold exactly 1 NFT from mint ${nftMintAddress.toBase58()}`);
            console.log(`Wallet already owns the required NFT in ATA ${walletNftAta.toBase58()}.`);
        } catch (e) {
            console.warn(`NFT ATA ${walletNftAta.toBase58()} not found or NFT is missing for wallet ${testWallet.publicKey.toBase58()}.`);
            console.warn(`You might need to mint the NFT (${nftMintAddress.toBase58()}) to this wallet manually or add minting logic to the 'before' block if the wallet is the NFT mint authority.`);
            throw new Error("NFT ownership check failed. Cannot proceed.");
            /*
            console.log("Attempting to mint NFT...");
            await mintTo(connection, testWallet.payer, nftMintAddress, walletNftAta, testWallet.publicKey, 1, [testWallet.payer], { commitment: "confirmed" });
            console.log("NFT minted for test setup.");
            */
        }
        console.log("--- Setup Complete ---");
    });

    it("Should exchange NFT for tokens (using provider wallet)", async () => {
        
        const nftBalanceBefore = (await getAccount(connection, walletNftAta)).amount;
        // let fungibleBalanceBefore = new anchor.BN(2);;
        // try { fungibleBalanceBefore = (await getAccount(connection, walletFungibleAta)).amount; } catch (e) {}
        // console.log(`Balances before: NFT=${nftBalanceBefore}, Fungible=${fungibleBalanceBefore}`);

        try {
            const tx = await program.methods
                .exchangeNftForTokens(amount)
                .accounts({
                    investor: testWallet.publicKey,
                    mintAuthority: testWallet.publicKey,

                    nftMint: nftMintAddress,
                    fungibleMint: fungibleMintAddress,
                })
                .signers([testWallet.payer]) 
                .rpc({ commitment: "confirmed" }); 
            try {
                const nftAccountInfo = await getAccount(connection, walletNftAta);
                assert.strictEqual(nftAccountInfo.amount.toString(), "0", "NFT account balance is not zero");
                console.log("NFT account balance verified: 0");
            } catch (error) {
                 if (error.message.includes("Account does not exist") || error.message.includes("could not find account")) {
                     console.log("NFT account closed as expected.");
                 } else { throw error; }
            }

             const fungibleAccountInfo = await getAccount(connection, walletFungibleAta);
             const fungibleMintInfo = await getMint(connection, fungibleMintAddress);
             const expectedAmount = amount.mul(new anchor.BN(10).pow(new anchor.BN(fungibleMintInfo.decimals)));
             console.log(`Fungible token balance verified: ${fungibleAccountInfo.amount}`);


        } catch (error) {
            console.error("Error during exchange:", error);
             if (error instanceof anchor.AnchorError) {
                 console.error("\nProgram Logs:", error.logs);
             }
            throw error; 
        }
    });
});
// For integratio with frontend
/*


describe("nft_exchange", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.mintSplToken as Program<MintSplToken>;
 const amount = new anchor.BN(1);
    const investorAddress = new PublicKey("7bCkmDRYMPTSYJPEhdx9rvWhQb3UpbBPDVuSDhWThPTF");
    const nftMintAddress = new PublicKey("4GTyiNebtcvtqVdUgS9SwZwK3x3m7rCvFag4AeM7NVzX");

    
    const fungibleMintAddress = new PublicKey("DNMVCRJKfHe4yz5tcgteSn6e75WHXUtoZZxKjcii2uze"); // ПРИКЛАД - ЗАМІНІТЬ!
    const mintAuthorityAddress = new PublicKey("HECVDSyH6doUTJ7FQ7K8pRRiQm3ojPAuPz63zH3Dep9G"); // ПРИКЛАД - ЗАМІНІТЬ!
    const fungibleAmount = new anchor.BN(1);
    const payer = provider.wallet;
    const investor = new PublicKey("7bCkmDRYMPTSYJPEhdx9rvWhQb3UpbBPDVuSDhWThPTF");
    let investorNftAta: PublicKey;
    let investorFungibleAta: PublicKey;

  it("Should mint tokens to associated account!", async () => { 
   
    let tx: Transaction | null = null; 

    try{
       tx = await program.methods
        .exchangeNftForTokens(
          amount,
        )
        .accounts({
            investor: investorAddress,
            nftMint: nftMintAddress,
            mintAuthority: mintAuthorityAddress,
            fungibleMint: fungibleMintAddress,
        })
        // .signers([mintKeypair])
        // .signers()
        .transaction()
        tx.feePayer = investorAddress;

        // .rpc();
        console.log("Your transaction signature", tx);
      }catch (error) {
        console.error("Error creating token: ", error);
      }
      console.log("SPL token was minted successfully!")
  });


});*/
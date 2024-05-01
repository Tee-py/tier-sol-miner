import * as anchor from "@coral-xyz/anchor";
import { TOKEN_DECIMALS } from "./constants";
import {  
    getMintToAddressInstruction,
    getKeypair
} from "./shared"

const mintTokens = async () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const connection = provider.connection;
    const adminKeyPair = getKeypair("deployer");
    const mintKeyPair = getKeypair("mint");
    
    // Token Minting
    console.log("Started Token Minting For Test Accounts");
    const mintToArgs = [
        // Deployer
        {
            account: new anchor.web3.PublicKey("8LkY6BDMrQKuqrJL5iR18Z6dmUKs45fFMtq8PbBUVZGj"),
            amount: 2000
        },
        // Devnet admin
        {
            account: new anchor.web3.PublicKey("2yZgY7sdYK31n1rifYBXBd3hCWeS1CzqYwv3Mzty82vo"),
            amount: 4000,
        },  
        // Presale Wallet     
        {
            account: new anchor.web3.PublicKey("FVHN3NdiUvfdzWRGji9uFzGALqSy7u2qF2zcwZcRTgmV"),
            amount: 1000,
        }
    ];
    const tokenMintTxn = new anchor.web3.Transaction();
    for (const arg of mintToArgs) {
        const Ix = await getMintToAddressInstruction(
            connection,
            adminKeyPair,
            mintKeyPair.publicKey,
            arg.account,
            arg.amount,
            TOKEN_DECIMALS
        )
        tokenMintTxn.add(Ix);
    }
    console.log("Submiting Token Mint Transaction...")
    await anchor.web3.sendAndConfirmTransaction(connection, tokenMintTxn, [adminKeyPair], undefined);
    console.log("Token Mint Transaction Completed Successfully ✅")
}

mintTokens()
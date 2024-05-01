import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TierSolMiner } from "../target/types/tier_sol_miner";
import { getKeypair } from "./shared";
import { TOKEN_DECIMALS } from "./constants";

interface TierInfo {
    apy: number,
    minimumTokenAmount: number,
    lockDuration: number,
    nonce: number,
}
const addTier = async (
    tierInfo: TierInfo[]
) => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const connection = provider.connection;
    const admin = provider.wallet;
    const adminKeyPair = getKeypair("deployer");

    const program = anchor.workspace.TierSolMiner as Program<TierSolMiner>;
    const [mineAccount, ] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("mine")], 
        program.programId
    );
    const addTierAccounts = {
        admin: admin.publicKey,
        mineInfo: mineAccount,
        systemProgram: anchor.web3.SystemProgram.programId  
    }
    const addTierTxn = new anchor.web3.Transaction();
    for (const info of tierInfo) {
        const [ tierAddress, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("tier"), Uint8Array.from([info.nonce])], 
            program.programId
        );
        console.log(`Creating Add Tier IX for Tier Address: ${tierAddress.toString()} and Nonce: ${info.nonce}`)
        const ix = await program.methods.addTier(
            new anchor.BN(info.apy),
            new anchor.BN(info.minimumTokenAmount),
            new anchor.BN(info.lockDuration)
        ).accounts({...addTierAccounts, tierInfo: tierAddress}).instruction();
        addTierTxn.add(ix);
    };
    console.log("Submitting Add Tier Transaction...");
    const sig = await anchor.web3.sendAndConfirmTransaction(connection, addTierTxn, [adminKeyPair], undefined);
    console.log(`Add Tier Transaction Completed ✅.\nSignature: ${sig}`)
}

addTier([
    {
        apy: 400000, // 4000% per year
        minimumTokenAmount: 0,
        lockDuration: 86400,
        nonce: 0
    },
    {
        apy: 800000, // 8000% per year
        minimumTokenAmount: 2000 * 10**TOKEN_DECIMALS,
        lockDuration: 172800,
        nonce: 1
    },
    {
        apy: 1600000, // 16000% per year
        minimumTokenAmount: 4000 * 10**TOKEN_DECIMALS,
        lockDuration: 604800,
        nonce: 2
    }
]);
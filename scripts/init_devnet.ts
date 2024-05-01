import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TierSolMiner } from "../target/types/tier_sol_miner";
import { 
    writePublicKey, writeSecretKey, 
    getKeypair, getMintSetUpInstruction,
    confirmTxn
} from "./shared"
import { TOKEN_DECIMALS, MAX_FEE, FEE_BASIS_POINTS } from "./constants";

const init = async (
    devFee: number,
    earlyClaimFee: number,
    referralReward: number
) => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const connection = provider.connection;
    const admin = provider.wallet;
    const adminKeyPair = getKeypair("deployer");
    let mintKeyPair: anchor.web3.Keypair;
    try {
        mintKeyPair = getKeypair("mint");
    } catch {
        // Mint SetUp Operations 
        console.log("Starting Mint Set Up Operation")
        mintKeyPair = new anchor.web3.Keypair();
        const setUpMintIxs = await getMintSetUpInstruction(
            connection,
            admin.publicKey,
            mintKeyPair,
            TOKEN_DECIMALS,
            BigInt(MAX_FEE),
            FEE_BASIS_POINTS
        )
        console.log("Submiting Mint SetUp Transaction")
        const setUpMintTxn = new anchor.web3.Transaction().add(...setUpMintIxs);
        await anchor.web3.sendAndConfirmTransaction(connection, setUpMintTxn, [adminKeyPair, mintKeyPair], undefined);
        console.log("mint set up transaction completed ✅")
        writePublicKey(mintKeyPair.publicKey, "mint");
        writeSecretKey(mintKeyPair.secretKey, "mint");
    };

    // Program Initialization Transaction
    const program = anchor.workspace.TierSolMiner as Program<TierSolMiner>;
    const [mineAccount, ] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("mine")], 
        program.programId
    );
    const [mineVault, ] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("mine-vault")], 
        program.programId
    );
    const initAccounts = {
        initializer: admin.publicKey,
        mineInfo: mineAccount,
        mineVault,
        systemProgram: anchor.web3.SystemProgram.programId
    };
    console.log("Sending Program Initialize Instruction...")
    const sig = await program.methods.initialize(
        admin.publicKey,
        admin.publicKey,
        mintKeyPair.publicKey,
        new anchor.BN(devFee),
        new anchor.BN(earlyClaimFee),
        new anchor.BN(referralReward)
    )
        .accounts({ ...initAccounts })
        .rpc()
    await confirmTxn(sig, connection);
}

init(
    500, // DevFee 5%
    4000, // Early Claim Fee 40%
    1000 // Referral Reward 10%
)
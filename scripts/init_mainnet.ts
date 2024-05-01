import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TierSolMiner } from "../target/types/tier_sol_miner";
import { 
    confirmTxn
} from "./shared"

const init = async (
    mint: anchor.web3.PublicKey,
    devFee: number,
    earlyClaimFee: number,
    referralReward: number
) => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const connection = provider.connection;
    const admin = provider.wallet;

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
        mint,
        new anchor.BN(devFee),
        new anchor.BN(earlyClaimFee),
        new anchor.BN(referralReward)
    )
        .accounts({ ...initAccounts })
        .rpc()
    await confirmTxn(sig, connection);
}

init(
    new anchor.web3.PublicKey(""), // Mint
    500, // DevFee 5%
    4000, // Early Claim Fee 40%
    1000 // Referral Reward 10%
)
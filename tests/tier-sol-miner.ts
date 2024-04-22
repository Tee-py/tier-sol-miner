import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TierSolMiner } from "../target/types/tier_sol_miner";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import { TOKEN_2022_PROGRAM_ID, ExtensionType, getMintLen, createInitializeMintInstruction, createInitializeTransferFeeConfigInstruction, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { expect } from "chai";

describe("tier-sol-miner", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const connection = anchor.getProvider().connection;

  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block
    });
    return signature
  }

  const calculateInterest = (amount: number, apy: number, interval: number) => {
    return Math.round((interval * amount * apy)/315_360_000_000)
  }

  const createMint = async () => {
    const extensions = [ExtensionType.TransferFeeConfig];
    const minLen = getMintLen(extensions);
    const requiredLamports = await connection.getMinimumBalanceForRentExemption(minLen);
    const mintTxn = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: initializer.publicKey,
        newAccountPubkey: mintKeyPair.publicKey,
        space: minLen,
        lamports: requiredLamports,
        programId: TOKEN_2022_PROGRAM_ID
      }),
      createInitializeTransferFeeConfigInstruction(
        mintKeyPair.publicKey,
        initializer.publicKey,
        initializer.publicKey,
        800,
        BigInt(10000000),
        TOKEN_2022_PROGRAM_ID
      ),
      createInitializeMintInstruction(
        mintKeyPair.publicKey,
        TOKEN_DECIMALS,
        initializer.publicKey,
        null,
        TOKEN_2022_PROGRAM_ID
      )
    );
    await sendAndConfirmTransaction(connection, mintTxn, [initializer, mintKeyPair]);
  }
  const mintToAccount = async (account: PublicKey, amount: number) => {
    const ata = await getOrCreateAssociatedTokenAccount(
      connection,
      initializer,
      mintKeyPair.publicKey,
      account,
      undefined,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    )
    await mintTo(
      connection,
      initializer,
      mintKeyPair.publicKey,
      ata.address,
      initializer,
      amount*10**TOKEN_DECIMALS,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    )
    return ata.address
  }
  const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

  const program = anchor.workspace.TierSolMiner as Program<TierSolMiner>;
  const TOKEN_DECIMALS = 6;
  const devFee = 10 // 0.1%;
  const earlyClaimFee = 8000 // 80%;
  const referralReward = 500 // 5%;
  const mintKeyPair = new Keypair();
  const initializer = new Keypair();
  const user1 = new Keypair();
  const user2 = new Keypair();
  const user3 = new Keypair();
  const user4 = new Keypair();
  const feeCollector = new Keypair();
  const penaltyCollector = new Keypair();
  const [mineAccount, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from("mine"), initializer.publicKey.toBuffer()], 
    program.programId
  );
  const [mineVault, ] = PublicKey.findProgramAddressSync(
    [Buffer.from("mine-vault"), initializer.publicKey.toBuffer()], 
    program.programId
  );
  const [ tier1, _bump1] = PublicKey.findProgramAddressSync(
    [Uint8Array.from([0]), initializer.publicKey.toBuffer()], 
    program.programId
  );
  const [ tier2, _bump2] = PublicKey.findProgramAddressSync(
    [Uint8Array.from([1]), initializer.publicKey.toBuffer()], 
    program.programId
  );
  const [ tier3, _bump3] = PublicKey.findProgramAddressSync(
    [Uint8Array.from([2]), initializer.publicKey.toBuffer()], 
    program.programId
  );
  const tierInfo = {
    tier1: {
      apy: 2000,
      minimumTokenAmount: 0,
      lockDuration: 5,
      tierAddress: tier1
    },
    tier2: {
      apy: 4000,
      minimumTokenAmount: 1000,
      lockDuration: 5,
      tierAddress: tier2
    },
    tier3: {
      apy: 6000,
      minimumTokenAmount: 2000,
      lockDuration: 5,
      tierAddress: tier3
    }
  }
  before(async () => {
    // load initializer with sol
    const sig1 = await connection.requestAirdrop(initializer.publicKey, 100*LAMPORTS_PER_SOL);
    await confirm(sig1)

    // Create mint
    await createMint();

    // Initialize program
    const accounts = {
      initializer: initializer.publicKey,
      mineInfo: mineAccount,
      mineVault,
      systemProgram: SystemProgram.programId
    };
    await program.methods.initialize(
      feeCollector.publicKey,
      penaltyCollector.publicKey,
      mintKeyPair.publicKey,
      new anchor.BN(devFee),
      new anchor.BN(earlyClaimFee),
      new anchor.BN(referralReward)
    )
      .accounts({ ...accounts })
      .signers([initializer])
      .rpc()
      .then(confirm)

    // Add Tiers
    const addTierAccounts = {
      admin: initializer.publicKey,
      mineInfo: mineAccount,
      systemProgram: SystemProgram.programId
    }
    const tierArray = ["tier1", "tier2", "tier3"]
    for (const tier of tierArray) {
      await program.methods.addTier(
        new anchor.BN(tierInfo[tier].apy),
        new anchor.BN(tierInfo[tier].minimumTokenAmount),
        new anchor.BN(tierInfo[tier].lockDuration),
      )
        .accounts({ ...addTierAccounts, tierInfo: tierInfo[tier].tierAddress})
        .signers([initializer])
        .rpc()
        .then(confirm)
    }

    // Load user accounts with SOL
    const user1Sig = await connection.requestAirdrop(user1.publicKey, 100*LAMPORTS_PER_SOL);
    const user2Sig = await connection.requestAirdrop(user2.publicKey, 100*LAMPORTS_PER_SOL);
    const user3Sig = await connection.requestAirdrop(user3.publicKey, 100*LAMPORTS_PER_SOL);
    const user4Sig = await connection.requestAirdrop(user4.publicKey, 100*LAMPORTS_PER_SOL);
    await confirm(user1Sig);
    await confirm(user2Sig);
    await confirm(user3Sig);
    await confirm(user4Sig);
  })

  it("Mine Information Verification", async () => {
    const mineInfo = await program.account.mineInfo.fetch(mineAccount);
    expect(mineInfo.admin.toString()).to.equals(initializer.publicKey.toString());
    expect(mineInfo.tokenMint.toString()).to.equals(mintKeyPair.publicKey.toString());
    expect(mineInfo.feeCollector.toString()).to.equals(feeCollector.publicKey.toString());
    expect(mineInfo.penaltyFeeCollector.toString()).to.equals(penaltyCollector.publicKey.toString());
    expect(mineInfo.devFee.toNumber()).to.equals(devFee);
    expect(mineInfo.earlyWithdrawalFee.toNumber()).to.equals(earlyClaimFee);
    expect(mineInfo.referralReward.toNumber()).to.equals(referralReward);
    expect(mineInfo.bump).to.equals(bump);
    expect(mineInfo.currentTierNonce.toString()).to.equals("3");
    expect(mineInfo.isActive).to.equals(true);
  })

  it("Tier Info Verification", async () => {
    const tier1Info = await program.account.tierInfo.fetch(tier1);
    const mineInfo = await program.account.mineInfo.fetch(mineAccount);
    const expectedInfo = tierInfo["tier1"];
    expect(tier1Info.apy.toNumber()).to.equals(expectedInfo.apy);
    expect(tier1Info.minimumTokenAmount.toNumber()).to.equals(expectedInfo.minimumTokenAmount);
    expect(tier1Info.totalLocked.toNumber()).to.equals(0);
    expect(tier1Info.lockDuration.toNumber()).to.equals(expectedInfo.lockDuration);
    expect(tier1Info.nonce).to.equals(0);
    expect(tier1Info.isActive).to.equals(true);
    expect(mineInfo.currentTierNonce).to.equals(3);
  });

  it("Init Staking Test [No Referrer]", async () => {
    const [userInfoPK, _] = PublicKey.findProgramAddressSync([Buffer.from("user"), user1.publicKey.toBuffer()], program.programId);
    const ata = await mintToAccount(user1.publicKey, 10);
    const accounts = {
      signer: user1.publicKey,
      userInfo: userInfoPK,
      tokenAccount: ata,
      mineInfo: mineAccount,
      mineVault,
      tierInfo: tier1,
      feeCollector: feeCollector.publicKey,
      systemProgram: SystemProgram.programId
    }
    const tierInfo = await program.account.tierInfo.fetch(tier1);
    const beforeInitVaultBalance = await connection.getBalance(mineVault);
    const beforeInitFeeCollectorBalance = await connection.getBalance(feeCollector.publicKey);
    await program.methods.initializeStaking(
      tierInfo.nonce,
      new anchor.BN(10*LAMPORTS_PER_SOL)
    )
      .accounts({ ...accounts })
      .signers([user1])
      .rpc()
      .then(confirm)
    const afterInitTierInfo = await program.account.tierInfo.fetch(tier1);
    const userInfo = await program.account.userInfo.fetch(userInfoPK);
    const afterInitVaultBalance = await connection.getBalance(mineVault);
    const afterInitFeeCollectorBalance = await connection.getBalance(feeCollector.publicKey);

    const expectedDevFee = (devFee * 10 * LAMPORTS_PER_SOL)/10000;
    const expectedTotalLocked = (10 * LAMPORTS_PER_SOL) - expectedDevFee;
    const expectedAccruedInterest = calculateInterest(expectedTotalLocked, tierInfo.apy.toNumber(), tierInfo.lockDuration.toNumber());

    expect(userInfo.owner.toString()).to.equals(user1.publicKey.toString());
    expect(userInfo.totalLocked.toString()).to.equals(expectedTotalLocked.toString());
    expect(expectedAccruedInterest - userInfo.accruedInterest.toNumber()).to.equals(2); // Must not be more than 2 LAMPORTS
    expect(userInfo.lockTs.toNumber()).to.greaterThan(1000000);
    expect(userInfo.tier.toString()).to.equals(tier1.toString());
    expect(userInfo.isWhitelist).to.equals(false);
    expect(afterInitVaultBalance - beforeInitVaultBalance).to.equals(expectedTotalLocked);
    expect(afterInitFeeCollectorBalance - beforeInitFeeCollectorBalance).to.equals(expectedDevFee);
    expect(afterInitTierInfo.totalLocked.toNumber() - tierInfo.totalLocked.toNumber()).to.equals(expectedTotalLocked);
  });

  it("Init Staking Test [Referrer]", async () => {
    //console.log("init test")
  });

  it("Init Staking Test [Insufficient Token Balance]", async () => {
    //console.log("init test")
  });

  it("Init Staking Test [Wrong Mint Address]", async () => {
    //console.log("init test")
  });

  it("Init Staking Test [Wrong Token Account Owner]", async () => {
    //console.log("init test")
  });

  it("Init Staking Test [Invalid Referrer]", async () => {
    //console.log("init test")
  });

  it("Init Staking Test [Invalid Fee Collector]", async () => {
    //console.log("init test")
  });

  it("Increase stake Test", async () => {
    //console.log("init test")
  });

  it("Whitelist deposit Test", async () => {
    //console.log("init test")
  });
});

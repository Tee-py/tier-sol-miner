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
    return Math.round(((interval * amount * apy)/315_360_000_000))
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
  const stakeAmount = 10; // 10 SOLs
  const devFee = 100 // 1%;
  const earlyClaimFee = 6000 // 60%;
  const referralReward = 1000 // 10%;
  const mintKeyPair = new Keypair();
  const initializer = new Keypair();
  const user1 = new Keypair();
  const user2 = new Keypair();
  const user3 = new Keypair();
  const user4 = new Keypair();
  const [user1InfoPk, _1] = PublicKey.findProgramAddressSync([Buffer.from("user"), user1.publicKey.toBuffer()], program.programId);
  const [user2InfoPk, _2] = PublicKey.findProgramAddressSync([Buffer.from("user"), user2.publicKey.toBuffer()], program.programId);
  const [user3InfoPk, _3] = PublicKey.findProgramAddressSync([Buffer.from("user"), user3.publicKey.toBuffer()], program.programId);
  const [user4InfoPk, _4] = PublicKey.findProgramAddressSync([Buffer.from("user"), user4.publicKey.toBuffer()], program.programId);
  const [user1refInfoPK, _5] = PublicKey.findProgramAddressSync([Buffer.from("referral"), user1InfoPk.toBuffer()], program.programId);
  // const [user2refInfoPK, _6] = PublicKey.findProgramAddressSync([Buffer.from("referral"), user2InfoPk.toBuffer()], program.programId);
  // const [user3refInfoPK, _7] = PublicKey.findProgramAddressSync([Buffer.from("referral"), user3InfoPk.toBuffer()], program.programId);
  // const [user4refInfoPK, _8] = PublicKey.findProgramAddressSync([Buffer.from("referral"), user4InfoPk.toBuffer()], program.programId);
  let user1Ata: PublicKey;
  let user2Ata: PublicKey;
  let user3Ata: PublicKey;
  let user4Ata: PublicKey;
  const feeCollector = new Keypair();
  const penaltyCollector = new Keypair();
  const [mineAccount, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from("mine")], 
    program.programId
  );
  const [mineVault, ] = PublicKey.findProgramAddressSync(
    [Buffer.from("mine-vault")], 
    program.programId
  );
  const [ tier1, bump1] = PublicKey.findProgramAddressSync(
    [Buffer.from("tier"), Uint8Array.from([0])], 
    program.programId
  );
  const [ tier2, bump2] = PublicKey.findProgramAddressSync(
    [Buffer.from("tier"), Uint8Array.from([1])], 
    program.programId
  );
  const [ tier3, bump3] = PublicKey.findProgramAddressSync(
    [Buffer.from("tier"), Uint8Array.from([2])], 
    program.programId
  );
  const tierInfo = {
    tier1: {
      apy: 31536000000, // 10% per second
      minimumTokenAmount: 0,
      lockDuration: 5,
      tierAddress: tier1,
      nonce: 0,
      bump: bump1
    },
    tier2: {
      apy: 63072000000, // 20% per second
      minimumTokenAmount: 1000 * 10**TOKEN_DECIMALS,
      lockDuration: 5,
      tierAddress: tier2,
      nonce: 1,
      bump: bump2
    },
    tier3: {
      apy: 94608000000, // 30% per second
      minimumTokenAmount: 2000 * 10**TOKEN_DECIMALS,
      lockDuration: 5,
      tierAddress: tier3,
      nonce: 2,
      bump: bump3
    }
  }
  const tierArray = ["tier1", "tier2", "tier3"];
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

    // Load mine vault with SOL
    const vaultSig = await connection.requestAirdrop(mineVault, 1000*LAMPORTS_PER_SOL);
    await confirm(vaultSig);

    // Mint tokens to atas
    user1Ata = await mintToAccount(user1.publicKey, 10);
    user2Ata = await mintToAccount(user2.publicKey, 1000);
    user3Ata = await mintToAccount(user3.publicKey, 10);
    user4Ata = await mintToAccount(user4.publicKey, 10);
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
    for (const tier of tierArray) {
      const expectedInfo = tierInfo[tier];
      const [addr, _] = PublicKey.findProgramAddressSync([Buffer.from("tier"), Uint8Array.from([expectedInfo.nonce])], program.programId);
      const info = await program.account.tierInfo.fetch(expectedInfo.tierAddress);
      expect(info.apy.toNumber()).to.equals(expectedInfo.apy);
      expect(info.minimumTokenAmount.toNumber()).to.equals(expectedInfo.minimumTokenAmount);
      expect(info.totalLocked.toNumber()).to.equals(0);
      expect(info.lockDuration.toNumber()).to.equals(expectedInfo.lockDuration);
      expect(info.nonce).to.equals(expectedInfo.nonce);
      expect(info.isActive).to.equals(true);
      expect(expectedInfo.tierAddress.toString()).to.equals(addr.toString());
    }
    const mineInfo = await program.account.mineInfo.fetch(mineAccount);
    expect(mineInfo.currentTierNonce).to.equals(3);
  });

  it("Init Staking Test [No Referrer]", async () => {
    const accounts = {
      signer: user1.publicKey,
      userInfo: user1InfoPk,
      tokenAccount: user1Ata,
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
      new anchor.BN(stakeAmount*LAMPORTS_PER_SOL)
    )
      .accounts({ ...accounts })
      .signers([user1])
      .rpc()
      .then(confirm)
    const afterInitTierInfo = await program.account.tierInfo.fetch(tier1);
    const userInfo = await program.account.userInfo.fetch(user1InfoPk);
    const afterInitVaultBalance = await connection.getBalance(mineVault);
    const afterInitFeeCollectorBalance = await connection.getBalance(feeCollector.publicKey);

    const expectedDevFee = (devFee * stakeAmount * LAMPORTS_PER_SOL)/10000;
    const expectedTotalLocked = (stakeAmount * LAMPORTS_PER_SOL) - expectedDevFee;
    const expectedAccruedInterest = calculateInterest(expectedTotalLocked, tierInfo.apy.toNumber(), tierInfo.lockDuration.toNumber());

    expect(userInfo.owner.toString()).to.equals(user1.publicKey.toString());
    expect(userInfo.totalLocked.toString()).to.equals(expectedTotalLocked.toString());
    expect(expectedAccruedInterest).to.equals(userInfo.accruedInterest.toNumber());
    expect(userInfo.lockTs.toNumber()).to.greaterThan(1000000);
    expect(userInfo.tier.toString()).to.equals(tier1.toString());
    expect(userInfo.isWhitelist).to.equals(false);
    expect(afterInitVaultBalance - beforeInitVaultBalance).to.equals(expectedTotalLocked);
    expect(afterInitFeeCollectorBalance - beforeInitFeeCollectorBalance).to.equals(expectedDevFee);
    expect(afterInitTierInfo.totalLocked.toNumber() - tierInfo.totalLocked.toNumber()).to.equals(expectedTotalLocked);
  });

  it("Init Staking Test [Referrer]", async () => {
    const accounts = {
      signer: user2.publicKey,
      userInfo: user2InfoPk,
      tokenAccount: user2Ata,
      mineInfo: mineAccount,
      mineVault,
      tierInfo: tier2,
      referrerUserInfo: user1InfoPk,
      referrerInfo: user1refInfoPK,
      feeCollector: feeCollector.publicKey,
      systemProgram: SystemProgram.programId
    }
    const tierInfo = await program.account.tierInfo.fetch(tier2);
    const beforeInitVaultBalance = await connection.getBalance(mineVault);
    const beforeInitFeeCollectorBalance = await connection.getBalance(feeCollector.publicKey);
    await program.methods.initializeStakingWithReferrer(
      tierInfo.nonce,
      new anchor.BN(stakeAmount*LAMPORTS_PER_SOL)
    )
      .accounts({ ...accounts })
      .signers([user2])
      .rpc()
      .then(confirm)
    const afterInitTierInfo = await program.account.tierInfo.fetch(tier2);
    const userInfo = await program.account.userInfo.fetch(user2InfoPk);
    const afterInitVaultBalance = await connection.getBalance(mineVault);
    const afterInitFeeCollectorBalance = await connection.getBalance(feeCollector.publicKey);
    const referralInfo = await program.account.referralInfo.fetch(user1refInfoPK);

    const expectedDevFee = (devFee * stakeAmount * LAMPORTS_PER_SOL)/10000;
    const expectedTotalLocked = (stakeAmount * LAMPORTS_PER_SOL) - expectedDevFee;
    const expectedAccruedInterest = calculateInterest(expectedTotalLocked, tierInfo.apy.toNumber(), tierInfo.lockDuration.toNumber());
    const expectedReferralBonus = expectedTotalLocked * referralReward/10000;

    expect(userInfo.owner.toString()).to.equals(user2.publicKey.toString());
    expect(userInfo.totalLocked.toString()).to.equals(expectedTotalLocked.toString());
    expect(expectedAccruedInterest).to.equals(userInfo.accruedInterest.toNumber());
    expect(userInfo.lockTs.toNumber()).to.greaterThan(1000000);
    expect(userInfo.tier.toString()).to.equals(tier2.toString());
    expect(userInfo.isWhitelist).to.equals(false);
    expect(afterInitVaultBalance - beforeInitVaultBalance).to.equals(expectedTotalLocked);
    expect(afterInitFeeCollectorBalance - beforeInitFeeCollectorBalance).to.equals(expectedDevFee);
    expect(afterInitTierInfo.totalLocked.toNumber() - tierInfo.totalLocked.toNumber()).to.equals(expectedTotalLocked);

    expect(referralInfo.earnings.toNumber()).to.equals(expectedReferralBonus);
    expect(referralInfo.count.toNumber()).to.equals(1);
    expect(referralInfo.owner.toString()).to.equals(user1.publicKey.toString());
    expect(referralInfo.userInfo.toString()).to.equals(user1InfoPk.toString())
  });

  it("Increase stake Test", async () => {
    const accounts = {
      signer: user1.publicKey,
      userInfo: user1InfoPk,
      tokenAccount: user1Ata,
      mineInfo: mineAccount,
      mineVault,
      tierInfo: tier1,
      feeCollector: feeCollector.publicKey,
      systemProgram: SystemProgram.programId
    };
    const beforeIncrTierInfo = await program.account.tierInfo.fetch(tier1);
    const beforeIncrVaultBalance = await connection.getBalance(mineVault);
    const beforeIncrFeeCollectorBalance = await connection.getBalance(feeCollector.publicKey);
    const beforeIncrUserInfo = await program.account.userInfo.fetch(user1InfoPk);
    await sleep(5 * 1000); // Sleep for 5 seconds before increasing stake
    await program.methods.increaseStake(
      new anchor.BN(stakeAmount*LAMPORTS_PER_SOL)
    )
      .accounts({ ...accounts })
      .signers([user1])
      .rpc()
      .then(confirm)
    const expectedDevFee = (devFee * stakeAmount * LAMPORTS_PER_SOL)/10000;
    const actualDeposit = (stakeAmount * LAMPORTS_PER_SOL) - expectedDevFee;
    const minCurrentInterest = calculateInterest(beforeIncrUserInfo.totalLocked.toNumber(), tierInfo["tier1"].apy, 5);
    const maxCurrentInterest = calculateInterest(beforeIncrUserInfo.totalLocked.toNumber(), tierInfo["tier1"].apy, 7);
    const expectedNewTotalLocked = beforeIncrUserInfo.totalLocked.toNumber() + actualDeposit;
    const newInterest = calculateInterest(expectedNewTotalLocked, tierInfo["tier1"].apy, tierInfo["tier1"].lockDuration);
    const minExpectedInterestAccrued = newInterest + minCurrentInterest;
    const maxExpectedInterestAccrued = newInterest + maxCurrentInterest;
    const tInfo = await program.account.tierInfo.fetch(tier1);
    const afterIncrVaultBalance = await connection.getBalance(mineVault);
    const afterIncrFeeCollectorBalance = await connection.getBalance(feeCollector.publicKey);
    const userInfo = await program.account.userInfo.fetch(user1InfoPk);
    expect(userInfo.totalLocked.toNumber()).to.equals(expectedNewTotalLocked);
    expect(userInfo.accruedInterest.toNumber()).to.lessThanOrEqual(maxExpectedInterestAccrued);
    expect(userInfo.accruedInterest.toNumber()).to.greaterThanOrEqual(minExpectedInterestAccrued);
    expect(userInfo.lockTs.toNumber()).to.greaterThan(beforeIncrUserInfo.lockTs.toNumber());
    expect(tInfo.totalLocked.toNumber() - beforeIncrTierInfo.totalLocked.toNumber()).to.equals(actualDeposit);
    expect(afterIncrVaultBalance - beforeIncrVaultBalance).to.equals(actualDeposit);
    expect(afterIncrFeeCollectorBalance - beforeIncrFeeCollectorBalance).to.equals(expectedDevFee);
  });

  it("Account WhiteList Test", async () => {
    const [whitelistInfoPK, _] = PublicKey.findProgramAddressSync(
      [Buffer.from("whitelist"), user3.publicKey.toBuffer()], 
      program.programId
    );
    const accounts = {
      admin: initializer.publicKey,
      beneficiary: user3.publicKey,
      whitelistInfo: whitelistInfoPK,
      tierInfo: tier3,
      systemProgram: SystemProgram.programId
    }
    const tInfo = await program.account.tierInfo.fetch(tier3);
    const now = Date.now()/1000;
    const expiry = now + 10;
    await program.methods.whitelistAccount(
      tInfo.nonce,
      new anchor.BN(expiry)
    )
      .accounts({ ...accounts })
      .signers([initializer])
      .rpc()
      .then(confirm);

    const whitelistInfo = await program.account.whitelistInfo.fetch(whitelistInfoPK);
    expect(whitelistInfo.beneficiary.toString()).to.equals(user3.publicKey.toString());
    expect(whitelistInfo.tier.toString()).to.equals(tier3.toString());
    expect(whitelistInfo.expiry.toNumber()).to.equals(Math.floor(expiry));
  });

  it("Init WhiteList Test", async () => {
    const [whitelistInfoPK, _1] = PublicKey.findProgramAddressSync(
      [Buffer.from("whitelist"), user3.publicKey.toBuffer()], 
      program.programId
    );
    const accounts = {
      signer: user3.publicKey,
      userInfo: user3InfoPk,
      mineInfo: mineAccount,
      mineVault,
      whitelistInfo: whitelistInfoPK,
      tierInfo: tier3,
      feeCollector: feeCollector.publicKey,
      systemProgram: SystemProgram.programId
    };

    const tierInfo = await program.account.tierInfo.fetch(tier3);
    const beforeInitVaultBalance = await connection.getBalance(mineVault);
    const beforeInitFeeCollectorBalance = await connection.getBalance(feeCollector.publicKey);
    await program.methods.initializeWhitelist(
      tierInfo.nonce, 
      new anchor.BN(stakeAmount*LAMPORTS_PER_SOL)
    )
      .accounts({ ...accounts })
      .signers([user3])
      .rpc()
      .then(confirm)
    
    const afterInitTierInfo = await program.account.tierInfo.fetch(tier3);
    const userInfo = await program.account.userInfo.fetch(user3InfoPk);
    const afterInitVaultBalance = await connection.getBalance(mineVault);
    const afterInitFeeCollectorBalance = await connection.getBalance(feeCollector.publicKey);

    const expectedDevFee = (devFee * stakeAmount * LAMPORTS_PER_SOL)/10000;
    const expectedTotalLocked = (stakeAmount * LAMPORTS_PER_SOL) - expectedDevFee;
    const expectedAccruedInterest = calculateInterest(expectedTotalLocked, tierInfo.apy.toNumber(), tierInfo.lockDuration.toNumber());

    expect(userInfo.owner.toString()).to.equals(user3.publicKey.toString());
    expect(userInfo.totalLocked.toString()).to.equals(expectedTotalLocked.toString());
    expect(expectedAccruedInterest).to.equals(userInfo.accruedInterest.toNumber());
    expect(userInfo.lockTs.toNumber()).to.greaterThan(1000000);
    expect(userInfo.tier.toString()).to.equals(tier3.toString());
    expect(userInfo.isWhitelist).to.equals(true);
    expect(afterInitVaultBalance - beforeInitVaultBalance).to.equals(expectedTotalLocked);
    expect(afterInitFeeCollectorBalance - beforeInitFeeCollectorBalance).to.equals(expectedDevFee);
    expect(afterInitTierInfo.totalLocked.toNumber() - tierInfo.totalLocked.toNumber()).to.equals(expectedTotalLocked);
  });

  it("Interest Compounding Failure Test [Lock duration not reached]", async () => {
    const accounts = {
      signer: user1.publicKey,
      userInfo: user1InfoPk,
      tokenAccount: user1Ata,
      mineInfo: mineAccount,
      tierInfo: tier1
    };
    try {
      await program.methods.compound().accounts({...accounts}).signers([user1]).rpc().then(confirm)
    } catch (error) {
      expect(error).to.be.an('Error');
    }
  })

  it("Interest Compounding Success Test", async () => {
    const beforeCompUserInfo = await program.account.userInfo.fetch(user1InfoPk);
    const beforeCompTierInfo = await program.account.tierInfo.fetch(tier1);
    const accounts = {
      signer: user1.publicKey,
      userInfo: user1InfoPk,
      tokenAccount: user1Ata,
      mineInfo: mineAccount,
      tierInfo: tier1
    };
    await sleep(5*1000);
    await program.methods.compound()
      .accounts({...accounts})
      .signers([user1])
      .rpc()
      .then(confirm);
    const afterCompUserInfo = await program.account.userInfo.fetch(user1InfoPk);
    const afterCompTierInfo = await program.account.tierInfo.fetch(tier1);
    const expectedNewTotalLocked = beforeCompUserInfo.totalLocked.toNumber() + beforeCompUserInfo.accruedInterest.toNumber();
    const expectedNewInterest = calculateInterest(
      expectedNewTotalLocked, 
      beforeCompTierInfo.apy.toNumber(), 
      beforeCompTierInfo.lockDuration.toNumber()
    );
    expect(afterCompUserInfo.accruedInterest.toNumber()).to.equals(expectedNewInterest);
    expect(
      afterCompTierInfo.totalLocked.toNumber() - beforeCompTierInfo.totalLocked.toNumber()
    ).to.equals(beforeCompUserInfo.accruedInterest.toNumber());
    expect(afterCompUserInfo.totalLocked.toNumber()).to.equals(expectedNewTotalLocked);
    expect(afterCompUserInfo.lockTs.toNumber()).to.greaterThan(beforeCompUserInfo.lockTs.toNumber());
  })

  it("Claim Interest Tests [No Penalty]", async () => {
    await sleep(5*1000);
    const accounts = {
      signer: user1.publicKey,
      userInfo: user1InfoPk,
      tokenAccount: user1Ata,
      mineInfo: mineAccount,
      mineVault,
      tierInfo: tier1,
      feeCollector: feeCollector.publicKey,
      penaltyCollector: penaltyCollector.publicKey
    };
    const beforeClaimUserInfo = await program.account.userInfo.fetch(user1InfoPk);
    const beforeClaimUserBal = await connection.getBalance(user1.publicKey);
    const beforeClaimVaultBal = await connection.getBalance(mineVault);
    const beforeClaimFeeCollectorBal = await connection.getBalance(feeCollector.publicKey);
    const beforeClaimPenaltyCollectorBal = await connection.getBalance(penaltyCollector.publicKey);
    const expectedDevFee = (devFee * beforeClaimUserInfo.accruedInterest.toNumber())/10000;
    const expectedAmountOut = beforeClaimUserInfo.accruedInterest.toNumber() - expectedDevFee;
    const expectedNewInterest = calculateInterest(
      beforeClaimUserInfo.totalLocked.toNumber(), 
      tierInfo['tier1'].apy, tierInfo['tier1'].lockDuration
    );
    await program.methods.claimInterest()
      .accounts({...accounts})
      .signers([user1])
      .rpc({skipPreflight: true})
      .then(confirm)
    const afterClaimUserInfo = await program.account.userInfo.fetch(user1InfoPk);
    const afterClaimVaultBal = await connection.getBalance(mineVault);
    const afterClaimFeeCollectorBal = await connection.getBalance(feeCollector.publicKey);
    const afterClaimPenaltyCollectorBal = await connection.getBalance(penaltyCollector.publicKey);
    const afterClaimUserBal = await connection.getBalance(user1.publicKey);

    expect(afterClaimFeeCollectorBal - beforeClaimFeeCollectorBal).to.equals(expectedDevFee);
    expect(beforeClaimPenaltyCollectorBal).to.equals(afterClaimPenaltyCollectorBal);
    expect(beforeClaimVaultBal - afterClaimVaultBal).to.equals(beforeClaimUserInfo.accruedInterest.toNumber());
    expect(afterClaimUserBal - beforeClaimUserBal).to.equals(expectedAmountOut);
    expect(afterClaimUserInfo.lockTs.toNumber()).to.greaterThan(beforeClaimUserInfo.lockTs.toNumber());
    expect(afterClaimUserInfo.accruedInterest.toNumber()).to.equals(expectedNewInterest);
  })

  it("Claim Interest Tests [Penalty]", async () => {
    await sleep(3*1000);
    const accounts = {
      signer: user1.publicKey,
      userInfo: user1InfoPk,
      tokenAccount: user1Ata,
      mineInfo: mineAccount,
      mineVault,
      tierInfo: tier1,
      feeCollector: feeCollector.publicKey,
      penaltyCollector: penaltyCollector.publicKey
    };
    const beforeClaimUserInfo = await program.account.userInfo.fetch(user1InfoPk);
    const beforeClaimUserBal = await connection.getBalance(user1.publicKey);
    const beforeClaimVaultBal = await connection.getBalance(mineVault);
    const beforeClaimFeeCollectorBal = await connection.getBalance(feeCollector.publicKey);
    const beforeClaimPenaltyCollectorBal = await connection.getBalance(penaltyCollector.publicKey);
    const expectedDevFee = (devFee * beforeClaimUserInfo.accruedInterest.toNumber())/10000;
    const expectedPenaltyFee = (earlyClaimFee * beforeClaimUserInfo.accruedInterest.toNumber())/10000;
    const expectedAmountOut = beforeClaimUserInfo.accruedInterest.toNumber() - (expectedDevFee + expectedPenaltyFee);
    await program.methods.claimInterest()
      .accounts({...accounts})
      .signers([user1])
      .rpc({skipPreflight: true})
      .then(confirm)
    const afterClaimUserInfo = await program.account.userInfo.fetch(user1InfoPk);
    const afterClaimVaultBal = await connection.getBalance(mineVault);
    const afterClaimFeeCollectorBal = await connection.getBalance(feeCollector.publicKey);
    const afterClaimPenaltyCollectorBal = await connection.getBalance(penaltyCollector.publicKey);
    const afterClaimUserBal = await connection.getBalance(user1.publicKey);

    expect(afterClaimFeeCollectorBal - beforeClaimFeeCollectorBal).to.equals(expectedDevFee);
    expect(afterClaimPenaltyCollectorBal - beforeClaimPenaltyCollectorBal).to.equals(expectedPenaltyFee);
    expect(beforeClaimVaultBal - afterClaimVaultBal).to.equals(beforeClaimUserInfo.accruedInterest.toNumber());
    expect(afterClaimUserBal - beforeClaimUserBal).to.equals(expectedAmountOut);
    expect(afterClaimUserInfo.lockTs.toNumber()).to.greaterThan(beforeClaimUserInfo.lockTs.toNumber());
    expect(afterClaimUserInfo.accruedInterest.toNumber()).to.equals(0);
  })

  it("Referral Withdrawal Test", async () => {
    const accounts = {
      signer: user1.publicKey,
      userInfo: user1InfoPk,
      referrerInfo: user1refInfoPK,
      tokenAccount: user1Ata,
      tierInfo: tier1,
      mineInfo: mineAccount,
      mineVault,
      feeCollector: feeCollector.publicKey
    };
    const beforeClaimRefInfo = await program.account.referralInfo.fetch(user1refInfoPK);
    const beforeClaimUserBal = await connection.getBalance(user1.publicKey);
    const beforeClaimVaultBal = await connection.getBalance(mineVault);
    const beforeClaimFeeCollectorBal = await connection.getBalance(feeCollector.publicKey);
    const expectedDevFee = (beforeClaimRefInfo.earnings.toNumber() * devFee)/10000;
    const expectedAmountOut = beforeClaimRefInfo.earnings.toNumber() - expectedDevFee;
    await program.methods.withdrawReferralRewards()
      .accounts({...accounts})
      .signers([user1])
      .rpc()
      .then(confirm);
    const afterClaimRefInfo = await program.account.referralInfo.fetch(user1refInfoPK);
    const afterClaimUserBal = await connection.getBalance(user1.publicKey);
    const afterClaimVaultBal = await connection.getBalance(mineVault);
    const afterClaimFeeCollectorBal = await connection.getBalance(feeCollector.publicKey);
    expect(afterClaimRefInfo.earnings.toNumber()).to.equals(0);
    expect(afterClaimUserBal - beforeClaimUserBal).to.equals(expectedAmountOut);
    expect(afterClaimFeeCollectorBal - beforeClaimFeeCollectorBal).to.equals(expectedDevFee);
    expect(beforeClaimVaultBal - afterClaimVaultBal).to.equals(beforeClaimRefInfo.earnings.toNumber());
  })

  it("Terminate Staking Test [With Referral Info]", async () => {
    const accounts = {
      admin: initializer.publicKey,
      userInfo: user1InfoPk,
      referrerInfo: user1refInfoPK,
      mineInfo: mineAccount,
      mineVault,
      tierInfo: tier1,
      userAccount: user1.publicKey,
      feeCollector: feeCollector.publicKey,
      systemProgram: SystemProgram.programId
    };
    const userInfo = await program.account.userInfo.fetch(user1InfoPk);
    const beforeTmUserBal = await connection.getBalance(user1.publicKey);
    const beforeTmVaultBal = await connection.getBalance(mineVault);
    const beforeTmFeeCollectorBal = await connection.getBalance(feeCollector.publicKey);
    const expectedDevFee = (userInfo.totalLocked.toNumber() * devFee)/10000;
    const expectedAmountOut = userInfo.totalLocked.toNumber() - expectedDevFee;
    await program.methods.terminateStaking()
      .accounts({...accounts})
      .signers([initializer])
      .rpc()
      .then(confirm);
    const afterTmUserBal = await connection.getBalance(user1.publicKey);
    const afterTmVaultBal = await connection.getBalance(mineVault);
    const afterTmFeeCollectorBal = await connection.getBalance(feeCollector.publicKey);
    expect(afterTmUserBal - beforeTmUserBal).to.equals(expectedAmountOut);
    expect(afterTmFeeCollectorBal - beforeTmFeeCollectorBal).to.equals(expectedDevFee);
    expect(beforeTmVaultBal - afterTmVaultBal).to.equals(userInfo.totalLocked.toNumber());
    try {
      await program.account.userInfo.fetch(user1InfoPk);
    } catch (error) {
      expect(error).to.be.an('Error');
    }
    try {
      await program.account.referralInfo.fetch(user1refInfoPK);
    } catch (error) {
      expect(error).to.be.an('Error');
    }
  })

  it("Terminate Staking Test [No Referral Info]", async () => {
    const accounts = {
      admin: initializer.publicKey,
      userInfo: user2InfoPk,
      referrerInfo: null,
      mineInfo: mineAccount,
      mineVault,
      tierInfo: tier2,
      userAccount: user2.publicKey,
      feeCollector: feeCollector.publicKey,
      systemProgram: SystemProgram.programId
    };
    const userInfo = await program.account.userInfo.fetch(user2InfoPk);
    const beforeTmUserBal = await connection.getBalance(user2.publicKey);
    const beforeTmVaultBal = await connection.getBalance(mineVault);
    const beforeTmFeeCollectorBal = await connection.getBalance(feeCollector.publicKey);
    const expectedDevFee = (userInfo.totalLocked.toNumber() * devFee)/10000;
    const expectedAmountOut = userInfo.totalLocked.toNumber() - expectedDevFee;
    await program.methods.terminateStaking()
      .accounts({...accounts})
      .signers([initializer])
      .rpc()
      .then(confirm);
    const afterTmUserBal = await connection.getBalance(user2.publicKey);
    const afterTmVaultBal = await connection.getBalance(mineVault);
    const afterTmFeeCollectorBal = await connection.getBalance(feeCollector.publicKey);
    expect(afterTmUserBal - beforeTmUserBal).to.equals(expectedAmountOut);
    expect(afterTmFeeCollectorBal - beforeTmFeeCollectorBal).to.equals(expectedDevFee);
    expect(beforeTmVaultBal - afterTmVaultBal).to.equals(userInfo.totalLocked.toNumber());
    try {
      await program.account.userInfo.fetch(user1InfoPk);
    } catch (error) {
      expect(error).to.be.an('Error');
    }
  })

  it("Update Tier Test", async () => {
    const accounts = {
      admin: initializer.publicKey,
      tierInfo: tier2,
      mineInfo: mineAccount
    }
    const newApy = 3000;
    const newLockDuration = 5000;
    const newMinimumTokenAmount = 4000*10**TOKEN_DECIMALS;
    await program.methods.updateTier(
      new anchor.BN(newMinimumTokenAmount),
      new anchor.BN(newApy), // APY
      new anchor.BN(newLockDuration), // Lock duration
      false
    )
      .accounts({...accounts})
      .signers([initializer])
      .rpc()
      .then(confirm);
    const tierInfo = await program.account.tierInfo.fetch(tier2);
    expect(tierInfo.apy.toNumber()).to.equals(newApy);
    expect(tierInfo.lockDuration.toNumber()).to.equals(newLockDuration);
    expect(tierInfo.minimumTokenAmount.toNumber()).to.equals(newMinimumTokenAmount);
    expect(tierInfo.isActive).to.equals(false);
  })

  it("Update Mine Test", async () => {
    const accounts = {
      admin: initializer.publicKey,
      mineInfo: mineAccount
    }
    await program.methods.updateMine(
      null, // Fee Collector
      null, // Penalty Collector
      null, // Dev Fee
      null, // Early Withdrawal Fee,
      null, // Referral Reward,
      false
    ).accounts({...accounts}).signers([initializer]).rpc().then(confirm);
    const mineInfo = await program.account.mineInfo.fetch(mineAccount);
    expect(mineInfo.isActive).to.equals(false);
  })
});

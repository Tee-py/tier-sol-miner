import {
    Connection,
    Keypair, PublicKey,
    SystemProgram,
    TransactionInstruction
} from "@solana/web3.js";
import {
    getOrCreateAssociatedTokenAccount,
    TOKEN_2022_PROGRAM_ID,
    ExtensionType,
    getMintLen,
    createInitializeTransferFeeConfigInstruction,
    createInitializeMintInstruction,
    createMintToInstruction
} from "@solana/spl-token";
import fs from "fs";
import {decode} from "bs58";

const BASE_KEY_PATH = "./.anchor/keys";
export const getPublicKey = (name: String) =>
    new PublicKey(
        JSON.parse(fs.readFileSync(`${BASE_KEY_PATH}/${name}_pub.json`) as unknown as string)
    );

export const getPrivateKey = (name: string) =>
    Uint8Array.from(
        JSON.parse(fs.readFileSync(`${BASE_KEY_PATH}/${name}.json`) as unknown as string)
    );

export const writePublicKey = (publicKey: PublicKey, name: string) => {
    const path = `${BASE_KEY_PATH}/${name}_pub.json`
    console.log(`Writing Public Key To: ${path}`)
    fs.writeFileSync(
        path,
        JSON.stringify(publicKey.toString())
    );
};
    
export const writeSecretKey = (secretKey: Uint8Array, name: string) => {
    const path = `${BASE_KEY_PATH}/${name}.json`
    console.log(`Writing Secret Key To: ${path}`)
    fs.writeFileSync(
        path,
        `[${secretKey.toString()}]`
    );
};

export const getKeypair = (name: string, isSecret?: boolean) => {
    if (isSecret) {
        const decoded = decode(JSON.parse(fs.readFileSync(`${BASE_KEY_PATH}/${name}.json`) as unknown as string));
        return Keypair.fromSecretKey(decoded);
    }
    return new Keypair({
        publicKey: getPublicKey(name).toBytes(),
        secretKey: getPrivateKey(name),
    });
}

export const getMintToAddressInstruction = async (
    connection: Connection,
    admin: Keypair,
    mint: PublicKey,
    to: PublicKey,
    amount: number,
    decimals: number
) => {
    const associatedToken = await getOrCreateAssociatedTokenAccount(
        connection,
        admin,
        mint,
        to,
        undefined,
        undefined,
        undefined,
        TOKEN_2022_PROGRAM_ID
    )
    console.log(`Creating Token Mint Instruction For ${associatedToken.address}`)
    return createMintToInstruction(
        mint,
        associatedToken.address,
        admin.publicKey,
        amount*10**decimals,
        [],
        TOKEN_2022_PROGRAM_ID
    )
}

export const getMintSetUpInstruction = async (
    connection: Connection,
    admin: PublicKey,
    mintKeyPair: Keypair,
    decimals: number,
    maxFee: bigint,
    feeBasisPoints: number
) => {
    let instructions: TransactionInstruction[] = [];
    const extensions = [
        ExtensionType.TransferFeeConfig
    ];
    const minLen = getMintLen(extensions);
    const requiredLamports = await connection.getMinimumBalanceForRentExemption(minLen);
    instructions.push(
        SystemProgram.createAccount({
            fromPubkey: admin,
            newAccountPubkey: mintKeyPair.publicKey,
            space: minLen,
            lamports: requiredLamports,
            programId: TOKEN_2022_PROGRAM_ID
        })
    )
    instructions.push(
        createInitializeTransferFeeConfigInstruction(
            mintKeyPair.publicKey,
            admin,
            admin,
            feeBasisPoints,
            maxFee,
            TOKEN_2022_PROGRAM_ID
        )
    )
    instructions.push(
        createInitializeMintInstruction(mintKeyPair.publicKey, decimals, admin, null, TOKEN_2022_PROGRAM_ID)
    )
    return instructions
};

export const confirmTxn = async (signature: string, connection: Connection): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block
    });
    console.log(`Txn Signature: ${signature}`)
    return signature
}
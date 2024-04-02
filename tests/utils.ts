import fs from "fs";
import { web3 } from "@coral-xyz/anchor";

export const lamportsToSol = (lamports: number) => lamports / web3.LAMPORTS_PER_SOL;
export const solToLamports = (sols: number) => web3.LAMPORTS_PER_SOL * sols;

export const loadKeypair = (walletPath: string): web3.Keypair => {
    try {
        const secretKey = new Uint8Array(
            JSON.parse(fs.readFileSync(walletPath).toString())
        );
        return web3.Keypair.fromSecretKey(secretKey);
    } catch (error) {
        console.error(`Failed to read ${walletPath} json file`);
        throw error;
    }
}

export const generateStateAccount = async (connection: web3.Connection, stateFilePath: string): Promise<web3.Keypair> => {
    if(fs.existsSync(stateFilePath)) {
        return loadKeypair(stateFilePath);
    }

    const keypair = web3.Keypair.generate();
    const buffer = Array.from(keypair.secretKey);
    fs.writeFileSync("wallets/state.json", JSON.stringify(buffer), { encoding: "utf-8" });
    const transaction = new web3.Transaction().add(
        web3.SystemProgram.createAccount({
            fromPubkey: keypair.publicKey,
            newAccountPubkey: keypair.publicKey,
            lamports: await connection.getMinimumBalanceForRentExemption(1024),
            space: 1024,
            programId: new web3.PublicKey('5Ta8DofvfQ8FoJvwjApYe7jbXqqwT4UpXrBXBX3eTVxz')
        })
    );

    await web3.sendAndConfirmTransaction(connection, transaction, [keypair]);
    await connection.requestAirdrop(keypair.publicKey, solToLamports(1));
    return keypair;
}

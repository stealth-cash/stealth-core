import fs from "fs";
import { web3 } from "@coral-xyz/anchor";

export type Net = "devnet" | "testnet" | "mainnet";

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

export const generateStateAccount = async (
    connection: web3.Connection,
    stateFilePath: string,
    developerKeypair: web3.Keypair // developer keypair
): Promise<web3.Keypair> => {
    if(fs.existsSync(stateFilePath)) {
        return loadKeypair(stateFilePath);
    }

    const keypair = web3.Keypair.generate();

    const tx = new web3.Transaction().add(
        // Create an account
        web3.SystemProgram.createAccount({
            fromPubkey: developerKeypair.publicKey,
            newAccountPubkey: keypair.publicKey,
            lamports: solToLamports(0.5),
            space: 1024,
            programId: new web3.PublicKey("GZFcqq4j4ptgHMVnFk8t4hDxCRS5Rrt1aNCBNj4hX3Lt")
        })
    );

    try {
        const sig = await web3.sendAndConfirmTransaction(connection, tx, [developerKeypair, keypair]);
        const buffer = Array.from(keypair.secretKey);
        fs.writeFileSync("wallets/state.json", JSON.stringify(buffer), { encoding: "utf-8" });  
        console.log("Created state account and transferred 0.5 SOL", sig);
    } catch (error) {
        console.error("Could not confirm tx", error);
    }

    return keypair;
}
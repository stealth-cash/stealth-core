import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StealthCash } from "../target/types/stealth_cash";
import fs from "fs";
import { assert } from "chai";

const loadKeypair = (walletPath: string): anchor.web3.Keypair => {
    try {
        const secretKey = new Uint8Array(
            JSON.parse(fs.readFileSync(walletPath).toString())
        );
        return anchor.web3.Keypair.fromSecretKey(secretKey);
    } catch (error) {
        console.error(`Failed to read ${walletPath} json file`);
        throw error;
    }
}

const lamportsToSol = (lamports: number) => lamports / anchor.web3.LAMPORTS_PER_SOL;

const generateStateAccount = (): anchor.web3.Keypair => {
    const stateFilePath = "wallets/state.json";
    let keypair: anchor.web3.Keypair;
    if(!fs.existsSync(stateFilePath)) {
        keypair = anchor.web3.Keypair.generate();
        const buffer = Array.from(keypair.secretKey);
        fs.writeFileSync("wallets/state.json", JSON.stringify(buffer), { encoding: "utf-8" });
    } else {
        keypair = loadKeypair(stateFilePath);
    }
    return keypair;
}

describe("stealth-cash", async () => {
    anchor.setProvider(anchor.AnchorProvider.env());

    const { connection } = anchor.getProvider();
    const program = anchor.workspace.StealthCash as Program<StealthCash>;
    const developerKeypair = loadKeypair("wallets/w1.json");
    const stateAccountKeypair = generateStateAccount();
    
    it("Testing keypairs", () => {
        assert(!!developerKeypair, "Developer keypair is missing");
        assert(!!stateAccountKeypair, "State account keypair is missing");
    });

    it("Checking balance", async () => {
        const lamports = await connection.getBalance(developerKeypair.publicKey);
        const sols = lamportsToSol(lamports);
        assert(sols >= 2, "balance is less that 2");
    });

    //  TODO: Fix
    //! Not initializing  
    // it("Is initialized!", async () => {
    //     const info = await connection.getBalance(developerKeypair.publicKey, "confirmed");
    //     console.log(info);
    //     const tx = await program.methods
    //         .initialize(new anchor.BN(100), 32)
    //         .accounts({ state: stateAccountKeypair.publicKey })
    //         .rpc();


    //     console.log("Your transaction signature", tx);
    //     assert(!!tx, "State does not exist");
    // });
});

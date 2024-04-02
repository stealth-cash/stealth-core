import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StealthCash } from "../target/types/stealth_cash";
import { assert } from "chai";
import { loadKeypair, generateStateAccount, lamportsToSol } from "./utils";


describe("stealth-cash", async () => {
    anchor.setProvider(anchor.AnchorProvider.env());

    const { connection } = anchor.getProvider();
    const program = anchor.workspace.StealthCash as Program<StealthCash>;
    const developerKeypair = loadKeypair("wallets/w1.json");
    const stateAccountKeypair = await generateStateAccount(connection, "wallets/state.json", developerKeypair);
    
    it("Testing keypairs", () => {
        assert(!!developerKeypair, "Developer keypair is missing");
        assert(!!stateAccountKeypair, "State account keypair is missing");
    });

    it("Checking balance", async () => {
        const lamports = await connection.getBalance(developerKeypair.publicKey);
        const sols = lamportsToSol(lamports);
        assert(sols >= 1, "balance is less that 2");
    });

    it("Is initialized!", async () => {
        const ix = await program.methods
            .initialize(new anchor.BN(100), 32)
            .accounts({ state: stateAccountKeypair.publicKey })
            .instruction();
        
        await anchor.web3.sendAndConfirmTransaction(connection, new anchor.web3.Transaction().add(ix), [developerKeypair]);
        assert(true);
    });
});

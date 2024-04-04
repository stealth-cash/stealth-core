import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StealthCash } from "../target/types/stealth_cash";
import { assert } from "chai";
import { loadKeypair, generateStateAccount, lamportsToSol } from "./utils";

describe("Minimal Demo", async () => {
    anchor.setProvider(anchor.AnchorProvider.env());

    const { connection } = anchor.getProvider();
    const program = anchor.workspace.StealthCash as Program<StealthCash>;
    const developerKeypair = loadKeypair("wallets/w1.json");
    const stateAccountKeypair = await generateStateAccount(connection, "wallets/state.json", developerKeypair);

    it("Testing Deposit", async () => {
        
    });
});
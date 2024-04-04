import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StealthCash } from "../target/types/stealth_cash";
import { assert } from "chai";
import { loadKeypair, generateStateAccount, lamportsToSol, Net } from "../src/utils/solana";
import { Deposit, rbigint, toHex } from "../src/utils/zk";

const AMOUNT = 0.1;
const netId: Net = "devnet";

anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.StealthCash as Program<StealthCash>;

describe("stealth-cash", async () => {
    const { connection } = anchor.getProvider();
    const developerKeypair = loadKeypair("wallets/w1.json");
    const recipientKeypair = loadKeypair("wallets/recipient.json");
    const stateAccountKeypair = await generateStateAccount(connection, "wallets/state.json", developerKeypair);

    type Deposit = {
        commitment: string,
        leafIndex: number,
        timestamp: anchor.BN
    };
    
    const depositLogs = new Array<Deposit>();
    
    program.addEventListener("DepositEvent", (event, slot, sig) => {
        console.log("Slot", slot);
        console.log("Signature: ", sig);
        depositLogs.push({
            commitment: event.commitment,
            leafIndex: event.leafIndex,
            timestamp: event.timestamp
        });
    });

    const getPastEvents = () => depositLogs;

    const deposit = async () => {
        const deposit = new Deposit(rbigint(31), rbigint(31));
        const ix = await program.methods
            .deposit(toHex(deposit.commitment))
            .accounts({
                state: stateAccountKeypair.publicKey,
                sender: developerKeypair.publicKey,
                recipient: recipientKeypair.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId
            })
            .instruction();

        const tx = new anchor.web3.Transaction().add(ix);
        try {
            const sig = await program.provider.sendAndConfirm(tx, [developerKeypair]);
            console.log(`https://solscan.io/tx/${sig}`);
            assert(true);          
            return `stealth-sol-${AMOUNT}-${netId}-${toHex(deposit.preimage, 62)}`;
        } catch (error) {
            console.error(error);
            assert(false, error);
        }
    };

    // it("Testing keypairs", () => {
    //     assert(!!developerKeypair, "Developer keypair is missing");
    //     assert(!!stateAccountKeypair, "State account keypair is missing");
    // });

    // it("Checking balance", async () => {
    //     const lamports = await connection.getBalance(developerKeypair.publicKey);
    //     const sols = lamportsToSol(lamports);
    //     assert(sols >= 1, "balance is less that 2");
    // });

    // it("Initialize state", async () => {
    //     const ix = await program.methods
    //         .initialize(new anchor.BN(100), 32)
    //         .accounts({ 
    //             state: stateAccountKeypair.publicKey,
    //             payer: developerKeypair.publicKey,
    //             systemProgram: anchor.web3.SystemProgram.programId
    //         })
    //         .instruction();
    //     const sig = await anchor.web3.sendAndConfirmTransaction(
    //         connection, 
    //         new anchor.web3.Transaction().add(ix),
    //         [developerKeypair, stateAccountKeypair]
    //     );
    //     assert(!!sig, "Could not initialize state account");
    // });

    // it("Check Programs State", async () => {
    //     const accountClient = await program.account.state.all();
    //     console.log(accountClient);
    //     assert(true);
    // });

    it("Testing deposit", async () => {
        const note = await deposit();
        console.log("Note:", note);
        console.log("Past events: ", getPastEvents());
        assert(note.length > 50, "could not return note");
    });

});

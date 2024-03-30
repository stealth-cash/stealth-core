import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StealthCash } from "../target/types/stealth_cash";

describe("stealth-cash", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.StealthCash as Program<StealthCash>;

  it("Is initialized!", async () => {
    const tx = await program.methods.initialize(
      anchor.web3.Keypair.generate().publicKey,
      new anchor.BN(100),
      32    
    )
    .accounts({ state: anchor.web3.Keypair.generate().publicKey })
    .rpc();
    
    console.log("Your transaction signature", tx);
  });
});

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StealthCash } from "../target/types/stealth_cash";
import { loadKeypair, generateStateAccount, Net } from "./utils/solana";
import { Deposit, rbigint, toHex } from "./utils/zk";
import { bigInt } from "snarkjs";

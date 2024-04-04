import crypto from "node:crypto";
import circomlib from "circomlib";
import { bigInt } from "snarkjs";

type HashLike = Buffer | string | Buffer[] | string[];
type BigIntArgs = string | number | bigint | boolean;

export const rbigint = (nBytes: number) => bigInt.leBuff2int(crypto.randomBytes(nBytes));
export const pedersenHash = (data: HashLike): bigint => circomlib.babyJub.unpackPoint(circomlib.pedersenHash.hash(data))[0];
export const toHex = (n: BigIntArgs | Buffer, len = 32): string => 
    (n instanceof Buffer ? n.toString("hex") : bigInt(n).toString(16)).padStart(len * 2, "0");

export class Deposit {
    private readonly secret: bigInt;
    private readonly nullifier: bigInt;
    private readonly _preimage: Buffer;
    public readonly commitment: bigint; 
    public readonly nullifierHash: bigint;

    public constructor(
        secret: bigInt,
        nullifier: bigInt
    ) {
        this.secret = secret;
        this.nullifier = nullifier;
        this._preimage = Buffer.concat([nullifier.leInt2Buff(31), secret.leInt2Buff(31)]);
        this.commitment = pedersenHash(this._preimage);
        this.nullifierHash = pedersenHash(this.nullifier.leInt2Buff(31));
    }

    public get preimage() {
        return this._preimage;
    }
}
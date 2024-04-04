const { bigInt } = require("snarkjs");
const crypto = require("crypto");

const rbigint = (nBytes) => bigInt.leBuff2int(crypto.randomBytes(nBytes));

console.log(rbigint(31).leInt2Buff(31));
import { Parser } from "binary-parser";
import BN from "bn.js";

const leToBe = (arr: number[]) => {
  return Buffer.from(arr.reverse()).readUInt32BE(0);
};

export const PROOF_OF_BURN_SIG = Buffer.from([0xc5, 0xe1, 0x9c, 0x70]);
export const PROOF_OF_MINT_SIG = Buffer.from([0xab, 0xba, 0x24, 0x3b]);
export const APPROVED_BURN_PROOF_SIG = Buffer.from([0xa4, 0x39, 0xa6, 0x33]);

export enum ChainType {
  Undefined = 0,
  Evm,
  Casper,
  Solana,
  Radix,
}

export type ProofOfMint = {
  _length: 238;
  _sig: Buffer;
  mint_token: Buffer;
  burn_token: Buffer;
  mint_caller: Buffer;
  burn_caller: Buffer;
  burn_amount: BN;
  mint_chain_type: ChainType;
  mint_chain_id: number;
  burn_chain_type: ChainType;
  burn_chain_id: number;
  burn_proof_hash: Buffer;
};

const proofOfMint = new Parser()
  .endianess("big")
  .array("_length", {
    type: "uint8",
    length: 4,
    formatter: arr => {
      return leToBe(arr);
    },
  })
  .array("_sig", {
    type: "uint8",
    length: 4,
    formatter: arr => {
      return Buffer.from(arr, "hex");
    },
  })
  .array("mint_token", {
    type: "uint8",
    length: 40,
    formatter: arr => {
      return Buffer.from(arr, "hex");
    },
  })
  .array("burn_token", {
    type: "uint8",
    length: 40,
    formatter: arr => {
      return Buffer.from(arr, "hex");
    },
  })
  .array("mint_caller", {
    type: "uint8",
    length: 40,
    formatter: arr => {
      return Buffer.from(arr, "hex");
    },
  })
  .array("burn_caller", {
    type: "uint8",
    length: 40,
    formatter: arr => {
      return Buffer.from(arr, "hex");
    },
  })
  .array("burn_amount", {
    type: "uint8",
    length: 32,
    formatter: arr => {
      return new BN(arr);
    },
  })
  .uint8("mint_chain_type", {
    formatter: n => {
      return n as ChainType;
    },
  })
  .uint32("mint_chain_id")
  .uint8("burn_chain_type", {
    formatter: n => {
      return n as ChainType;
    },
  })
  .uint32("burn_chain_id")
  .array("burn_proof_hash", {
    type: "uint8",
    length: 32,
    formatter: arr => {
      return Buffer.from(arr, "hex");
    },
  });

export const parseProofOfMint = (buffer: Buffer) => {
  const result = proofOfMint.parse(buffer) as ProofOfMint;

  if (!result._sig.equals(PROOF_OF_MINT_SIG)) {
    throw new Error("invalid signature");
  }

  if (result._length !== 238) {
    throw new Error("invalid event");
  }

  return result;
};

export type ApprovedBurnProof = {
  _length: 36;
  _sig: Buffer;
  burn_proof_hash: Buffer;
};

const approvedBurnProof = new Parser()
  .endianess("big")
  .array("_length", {
    type: "uint8",
    length: 4,
    formatter: arr => {
      return leToBe(arr);
    },
  })
  .array("_sig", {
    type: "uint8",
    length: 4,
    formatter: arr => {
      return Buffer.from(arr, "hex");
    },
  })
  .array("burn_proof_hash", {
    type: "uint8",
    length: 32,
    formatter: arr => {
      return Buffer.from(arr, "hex");
    },
  });

export const parseApprovedBurnProof = (buffer: Buffer) => {
  const result = approvedBurnProof.parse(buffer) as ApprovedBurnProof;

  if (!result._sig.equals(APPROVED_BURN_PROOF_SIG)) {
    throw new Error("invalid signature");
  }

  if (result._length !== 36) {
    throw new Error("invalid event");
  }

  return result;
};

export type ProofOfBurn = {
  _length: 270;
  _sig: Buffer;
  mint_token: Buffer;
  burn_token: Buffer;
  mint_caller: Buffer;
  burn_caller: Buffer;
  burn_amount: BN;
  burn_nonce: BN;
  mint_chain_type: ChainType;
  mint_chain_id: number;
  burn_chain_type: ChainType;
  burn_chain_id: number;
  burn_proof_hash: Buffer;
};

const proofOfBurn = new Parser()
  .endianess("big")
  .array("_length", {
    type: "uint8",
    length: 4,
    formatter: arr => {
      return leToBe(arr);
    },
  })
  .array("_sig", {
    type: "uint8",
    length: 4,
    formatter: arr => {
      return Buffer.from(arr, "hex");
    },
  })
  .array("mint_token", {
    type: "uint8",
    length: 40,
    formatter: arr => {
      return Buffer.from(arr, "hex");
    },
  })
  .array("burn_token", {
    type: "uint8",
    length: 40,
    formatter: arr => {
      return Buffer.from(arr);
    },
  })
  .array("mint_caller", {
    type: "uint8",
    length: 40,
    formatter: arr => {
      return Buffer.from(arr);
    },
  })
  .array("burn_caller", {
    type: "uint8",
    length: 40,
    formatter: arr => {
      return Buffer.from(arr);
    },
  })
  .array("burn_amount", {
    type: "uint8",
    length: 32,
    formatter: arr => {
      return new BN(arr);
    },
  })
  .array("burn_nonce", {
    type: "uint8",
    length: 32,
    formatter: arr => {
      return new BN(arr);
    },
  })
  .uint8("mint_chain_type", {
    formatter: n => {
      return n as ChainType;
    },
  })
  .uint32("mint_chain_id")
  .uint8("burn_chain_type", {
    formatter: n => {
      return n as ChainType;
    },
  })
  .uint32("burn_chain_id")
  .array("burn_proof_hash", {
    type: "uint8",
    length: 32,
    formatter: arr => {
      return Buffer.from(arr);
    },
  });

export const parseProofOfBurn = (buffer: Buffer) => {
  const result = proofOfBurn.parse(buffer) as ProofOfBurn;

  if (!result._sig.equals(PROOF_OF_BURN_SIG)) {
    throw new Error("invalid signature");
  }

  if (result._length !== 270) {
    throw new Error("invalid event");
  }

  return result;
};

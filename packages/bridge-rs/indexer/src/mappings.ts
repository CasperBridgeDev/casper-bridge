import { BridgeTransfer, Proof } from ".prisma/client";
import { Decimal } from "@prisma/client/runtime";
import decimals from "decimal.js";
import { prisma } from "./db";
import {
  ApprovedBurnProof,
  ChainType,
  ProofOfBurn,
  ProofOfMint,
} from "./schema";

const getOrCreateBridgeTransfer = async (id: string) => {
  const transfer = await prisma.bridgeTransfer.upsert({
    where: { id },
    create: {
      id,
      status: "Created",
    },
    update: {},
  });

  return transfer;
};

const getOrCreateGlobal = () => {};

const evmAddress = (b: Buffer) => {
  return b.slice(20); // 40 - 20 = 20 bytes
};

const casperAddress = (b: Buffer) => {
  return b.slice(8); // 40 - 8 = 32 bytes
};

const updateBridgeTransfer = (input: BridgeTransfer) => {
  return prisma.bridgeTransfer.update({
    where: { id: input.id },
    data: input,
  });
};

const saveProof = (input: Proof) => {
  return prisma.proof.create({
    data: input,
  });
};

const deadAddress = Buffer.from([0x0, 0x0, 0x0, 0xde, 0xad]);

const convertAddress = (chainType: ChainType, b: Buffer) => {
  if (chainType === ChainType.Evm) {
    return "0x" + evmAddress(b).toString("hex");
  } else if (chainType === ChainType.Casper) {
    return "0x" + casperAddress(b).toString("hex");
  }

  // chain type not supported
  return "0x" + deadAddress.toString("hex");
};

export interface Event<T> {
  params: T;
  deployHash: string;
  blockNumber: number;
  timestamp: number;
}

export const handleProofOfBurn = async (event: Event<ProofOfBurn>) => {
  const {
    mint_chain_type,
    burn_chain_type,
    burn_proof_hash,
    burn_caller,
    mint_caller,
    burn_token,
    mint_token,
    burn_amount,
  } = event.params;

  const gasPrice = 0;
  const gasUsed = 0;

  const proofOfBurn = "0x" + burn_proof_hash.toString("hex");

  const bridgeTransfer = await getOrCreateBridgeTransfer(proofOfBurn);
  bridgeTransfer.status = "Burned";

  const amount = decimals.div(burn_amount.toString(), 1e18);

  const proof: Proof = {
    id: proofOfBurn,
    type: "Burn",
    nonce: event.params.burn_nonce.toNumber(), // safe, should not overflow: ;
    src: event.params.burn_chain_id,
    srcType: burn_chain_type,
    dest: event.params.mint_chain_id,
    destType: mint_chain_type,
    srcCaller: convertAddress(burn_chain_type, burn_caller),
    destCaller: convertAddress(mint_chain_type, mint_caller),
    srcToken: convertAddress(burn_chain_type, burn_token),
    destToken: convertAddress(mint_chain_type, mint_token),
    amount: new Decimal(amount.toString()),
    blockNumber: event.blockNumber,
    txHash: event.deployHash,
    timestamp: event.timestamp,
  };

  await updateBridgeTransfer(bridgeTransfer);
  await saveProof(proof);
};

export const handleProofOfMint = async (event: Event<ProofOfMint>) => {
  const {
    mint_chain_type,
    burn_chain_type,
    burn_proof_hash,
    burn_caller,
    mint_caller,
    burn_token,
    mint_token,
    burn_amount,
  } = event.params;

  const gasPrice = 0;
  const gasUsed = 0;

  const proofOfBurn = "0x" + burn_proof_hash.toString("hex");
  const bridgeTransfer = await getOrCreateBridgeTransfer(proofOfBurn);
  bridgeTransfer.status = "Executed";

  const amount = decimals.div(burn_amount.toString(), 1e18);

  const proof: Proof = {
    id: proofOfBurn,
    type: "Mint",
    nonce: null, // find from burn (?)
    src: event.params.burn_chain_id,
    srcType: burn_chain_type,
    dest: event.params.mint_chain_id,
    destType: mint_chain_type,
    srcCaller: convertAddress(burn_chain_type, burn_caller),
    destCaller: convertAddress(mint_chain_type, mint_caller),
    srcToken: convertAddress(burn_chain_type, burn_token),
    destToken: convertAddress(mint_chain_type, mint_token),
    amount: new Decimal(amount.toString()),
    blockNumber: event.blockNumber,
    txHash: event.deployHash,
    timestamp: event.timestamp,
  };

  await updateBridgeTransfer(bridgeTransfer);
  await saveProof(proof);
};

export const handleApprovedBurnProof = async (
  event: Event<ApprovedBurnProof>,
) => {
  const proofOfBurn = "0x" + event.params.burn_proof_hash.toString("hex");

  const bridgeTransfer = await getOrCreateBridgeTransfer(proofOfBurn);
  bridgeTransfer.status = "Approved";

  await updateBridgeTransfer(bridgeTransfer);
};

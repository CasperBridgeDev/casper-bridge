jmport { BridgeTransfer, Proof } from "../generated/schema";
import {
  ApprovedBurnProof,
  ProofOfBurn,
  ProofOfMint,
} from "../generated/SourceChain/CrossChainBridge";
import { BI_18, convertTokenToDecimal } from "./helpers";

function getOrCreateBridgeTransfer(id: string): BridgeTransfer | null {
  let transfer = BridgeTransfer.load(id);

  if (transfer == null) {
    transfer = new BridgeTransfer(id);
    transfer.status = "Created";
    transfer.save();
  }

  return transfer;
}

function evmAddress(b: string): string {
  return b.slice(2 + 20 * 2);
}

function casperAddress(b: string): string {
  return b.slice(2 + 8 * 2);
}

type i32 = number;

function convertAddress(chainType: i32, b: string): string {
  if (chainType === 1) {
    //evm
    return "0x" + evmAddress(b);
  } else if (chainType === 2) {
    // casper
    return "0x" + casperAddress(b);
  }

  // chain type not supported
  return "0x" + "000000dead";
}

export function handleProofOfBurn(event: ProofOfBurn): void {
  let mintChainType = event.params.mintChainType;
  let burnChainType = event.params.burnChainType;
  let burnProofHash = event.params.burnProofHash;
  let mintCaller = event.params.mintCaller;
  let burnCaller = event.params.burnCaller;
  let burnAmount = event.params.burnAmount;
  let burnToken = event.params.burnToken;
  let mintToken = event.params.mintToken;

  let proofHash = burnProofHash.toHexString();

  let bridgeTransfer = getOrCreateBridgeTransfer(proofHash);
  bridgeTransfer.status = "Burned";

  let burnedAmount = convertTokenToDecimal(burnAmount, BI_18);

  let proof = new Proof(proofHash);
  proof.type = "Burn";
  proof.nonce = event.params.burnNonce;
  proof.src = event.params.burnChainId.toI32();
  proof.srcType = event.params.burnChainType;
  proof.dest = event.params.mintChainId.toI32();
  proof.destType = event.params.mintChainType;
  proof.srcCaller = convertAddress(burnChainType, burnCaller.toHexString());
  proof.destCaller = convertAddress(mintChainType, mintCaller.toHexString());
  proof.srcToken = convertAddress(burnChainType, burnToken.toHexString());
  proof.destToken = convertAddress(mintChainType, mintToken.toHexString());
  proof.amount = burnedAmount;
  // proof.fee = amountAsFee;
  proof.txHash = event.transaction.hash;
  // proof.logIndex = event.logIndex;
  proof.blockNumber = event.block.number;
  proof.timestamp = event.block.timestamp;

  bridgeTransfer.save();
  proof.save();
}

export function handleProofOfMint(event: ProofOfMint): void {
  let mintChainType = event.params.mintChainType;
  let burnChainType = event.params.burnChainType;
  let burnProofHash = event.params.burnProofHash;
  let mintCaller = event.params.mintCaller;
  let burnCaller = event.params.burnCaller;
  let burnAmount = event.params.burnAmount;
  let burnToken = event.params.burnToken;
  let mintToken = event.params.mintToken;

  let proofHash = burnProofHash.toHexString();

  let bridgeTransfer = getOrCreateBridgeTransfer(proofHash);
  bridgeTransfer.status = "Executed";

  let mintedAmount = convertTokenToDecimal(burnAmount, BI_18);

  let proof = new Proof(proofHash);
  proof.type = "Mint";
  proof.nonce = null; // find from burn (?)
  proof.src = event.params.burnChainId.toI32();
  proof.srcType = event.params.burnChainType;
  proof.dest = event.params.mintChainId.toI32();
  proof.destType = event.params.mintChainType;
  proof.srcCaller = convertAddress(mintChainType, burnCaller.toHexString());
  proof.destCaller = convertAddress(burnChainType, mintCaller.toHexString());
  proof.srcToken = convertAddress(mintChainType, burnToken.toHexString());
  proof.destToken = convertAddress(burnChainType, mintToken.toHexString());
  proof.amount = mintedAmount;
  // proof.fee = amountAsFee;
  proof.txHash = event.transaction.hash;
  // proof.logIndex = event.logIndex;
  proof.blockNumber = event.block.number;
  proof.timestamp = event.block.timestamp;

  bridgeTransfer.save();
  proof.save();
}

export function handleApprovedBurnProof(event: ApprovedBurnProof): void {
  let proofHash = event.params.burnProofHash.toHexString();

  let bridgeTransfer = getOrCreateBridgeTransfer(proofHash);
  bridgeTransfer.status = "Approved";
  bridgeTransfer.save();
}

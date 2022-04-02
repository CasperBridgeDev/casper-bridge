import BN from "bn.js";
import {
  ApprovedBurnProof,
  ChainType,
  parseApprovedBurnProof,
  parseProofOfBurn,
  parseProofOfMint,
  ProofOfBurn,
  ProofOfMint,
  PROOF_OF_BURN_SIG,
} from "./schema";

test("test proof of burn", () => {
  const buffer = Buffer.from(
    "09010000c5e19c7000000000000000001a3d2469dabebb61a161cafd7eace57a630d26a04970e96c0e4b78d262c4ccc201010101010101010101010101010101010101010101010101010101010101010101010101010101000000000000000070d1c4dbdaa32b4f27351ce912fb6b877653ad14f30e5547b472dc27362f14740101010101010101010101010101010101010101010101010101010101010101010101010101010100000000000000000000000000000000000000000000000000000000000001f4000000000000000000000000000000000000000000000000000000000000000001000000014df33619aab3cca0a4d1a537443139dc79e5b4bd7edcaf1122d0540b892528a8",
    "hex",
  );
  const result = parseProofOfBurn(buffer);

  const bytes40 = Buffer.from(Array.from({ length: 40 }).map(_ => 1));

  // const mint_token = bytes40;
  // const mint_caller = bytes40;

  // TODO: compute burn hash

  expect(result).toBe({
    _length: 270,
    _sig: PROOF_OF_BURN_SIG,
    mint_token: expect.any(Buffer),
    burn_token: bytes40,
    mint_caller: expect.any(Buffer),
    burn_caller: bytes40,
    burn_proof_hash: Buffer.from([]),
    burn_amount: new BN(500),
    mint_chain_type: 2,
    mint_chain_id: 1010,
    burn_chain_id: 1,
    burn_nonce: new BN(0),
    burn_chain_type: ChainType.Evm,
  } as ProofOfBurn);
});

// TODO: implement
test("test approve burn proof", () => {
  const buffer = Buffer.from("000", "hex");
  const result = parseApprovedBurnProof(buffer);

  expect(result).toBe({
    // _length: 32,
  } as ApprovedBurnProof);
});

test("test proof of mint", () => {
  const buffer = Buffer.from("000", "hex");
  const result = parseProofOfMint(buffer);

  expect(result).toBe({
    // _length: 32,
  } as ProofOfMint);
});

import { PromisePool } from "@supercharge/promise-pool/dist/promise-pool";
import { CasperServiceByJsonRPC, GetDeployResult } from "casper-js-sdk";
import { prisma } from "./db";
import {
  handleApprovedBurnProof,
  handleProofOfBurn,
  handleProofOfMint,
} from "./mappings";
import {
  parseApprovedBurnProof,
  parseProofOfBurn,
  parseProofOfMint,
} from "./schema";
import {
  ExecutionResultSuccess,
  StoredContractByHash,
  SuperJsonBlock,
  TransformKey,
} from "./types";

const casperService = new CasperServiceByJsonRPC("http://localhost:11105/rpc");
// const casperService = new CasperServiceByJsonRPC(
//   "http://65.21.227.101:7777/rpc",
// );

const currentHeight = 4;

const CONTRACT_HASH =
  "842274145F6b250e7Be85FBE8435d108D6c5CAc95196cA218971e3B77f6caf17";

const parseTransformWith = <T>(
  some: TransformKey,
  parser: (b: Buffer) => T,
): T | [] => {
  if (!some.key.startsWith("uref-")) {
    return [];
  }

  const transform = some.transform;

  if (
    typeof transform === "object" &&
    "WriteCLValue" in transform &&
    typeof transform.WriteCLValue.cl_type === "object" &&
    transform.WriteCLValue.cl_type.List === "U8"
  ) {
    const buffer = Buffer.from(transform.WriteCLValue.bytes, "hex");
    return parser(buffer);
  }

  return [];
};

const toTimestamp = (date: string | number) => {
  if (typeof date === "number") {
    return date;
  }

  return Math.floor(Number(new Date(date)) / 1000);
};

export interface Event<T> {
  params: T;
  deployHash: string;
  blockNumber: number;
  timestamp: number;
}

const prepare = async (input: GetDeployResult, height: number) => {
  const deploy = input.deploy;

  console.log(input.deploy);
  // account
  // chainName
  const isOk = input.execution_results[0].result.Success;

  if (!isOk) {
    console.log("error while deploying");
    console.log(input.execution_results[0].result.Failure);
    // ignore error
    return [];
  }

  const deployInput = deploy.session as StoredContractByHash;

  // ignore module bytes output
  // @ts-ignore
  if (deploy.session?.ModuleBytes) {
    // @ts-ignore
    delete deploy.session?.ModuleBytes.module_bytes;
  }

  // @ts-ignore
  if (!deployInput.StoredContractByHash) {
    // if it does not exists, it's likely deployment of contract
    return [];
  }

  // @ts-ignore
  if (deployInput.StoredContractByHash.hash !== CONTRACT_HASH) {
    // ignore
    return [];
  }

  const executionResult = input.execution_results[0].result
    .Success as unknown as ExecutionResultSuccess;

  // @ts-ignore
  const entryPoint = deployInput.StoredContractByHash.entry_point;
  const transforms = executionResult.effect.transforms;

  console.log({ entryPoint });

  const createEvent = <T>(event: T): Event<T> => ({
    params: event,
    blockNumber: height,
    deployHash: deploy.hash,
    timestamp: toTimestamp(deploy.header.timestamp),
  });

  let events: Promise<void>[] = [];
  if (entryPoint === "burn_and_create_proof") {
    events = transforms
      .flatMap(some => parseTransformWith(some, parseProofOfBurn))
      .map(event => handleProofOfBurn(createEvent(event)));
  } else if (entryPoint === "mint_with_burn_proof") {
    events = transforms
      .flatMap(some => parseTransformWith(some, parseProofOfMint))
      .map(event => handleProofOfMint(createEvent(event)));
  } else if (entryPoint === "approve_burn_proof") {
    events = transforms
      .flatMap(some => parseTransformWith(some, parseApprovedBurnProof))
      .map(event => handleApprovedBurnProof(createEvent(event)));
  }

  return events;
};

// MUST BE SYNCRONIZED
const sync = async () => {
  const meta = await prisma.meta.findFirst({ where: { id: 0 } });

  // get max height
  // const maxHeight = Math.max(currentHeight, currentHeight);

  const maxHeight = Math.max(meta?.blockNumber!, meta?.blockNumber!);
  const block = await casperService.getBlockInfoByHeight(maxHeight);

  const { deploy_hashes, transfer_hashes } = (block.block as SuperJsonBlock)
    .body;

  if (maxHeight % 100 === 0) {
    console.log({
      maxHeight,
    });
  }

  if (deploy_hashes.length > 0) {
    console.log({
      deploy_hashes,
      maxHeight,
    });
  }

  const { results, errors } = await PromisePool.withConcurrency(10)
    .for(deploy_hashes)
    .process(async deployHash => {
      const currentHeight = 0;
      console.log("----------");
      return prepare(deploy, maxHeight);
    });

  for (const result of results) {
    for (const transform of result) {
      console.log("applied");
      await transform;
    }
  }

  // currentHeight++;
  await prisma.meta.update({
    where: { id: 0 },
    data: { blockNumber: maxHeight + 1 },
  });
};

const syncLoop = async () => {
  // bootstrap
  await prisma.meta.upsert({
    where: { id: 0 },
    create: {
      blockNumber: currentHeight,
      id: 0,
    },
    update: {
      // blockNumber: currentHeight,
    },
  });
  while (true) {
    try {
      await sync();
    } catch (error) {
      // await new Promise(r => setTimeout(r, 500));
    }
    // return;
  }
};

syncLoop();

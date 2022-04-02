import {
  CasperClient,
  CasperServiceByJsonRPC,
  CLByteArray,
  CLKey,
  CLList,
  CLPublicKey,
  CLU256,
  CLU32,
  CLU8,
  CLValue,
  DeployUtil,
  Keys,
  RuntimeArgs,
} from "casper-js-sdk";
import { Deploy } from "casper-js-sdk/dist/lib/DeployUtil";
import { AsymmetricKey, Secp256K1 } from "casper-js-sdk/dist/lib/Keys";
import fs from "fs";

const sleep = (ms: number) => {
  return new Promise(resolve => setTimeout(resolve, ms));
};

function randomSeed() {
  return Array.from({ length: 40 }, () => Math.floor(Math.random() * 128));
}

const getAccountFromKeyPair = (baseKeyPath: string) => {
  const privateKeyPath = baseKeyPath + "secret_key.pem";
  const publicKeyPath = baseKeyPath + "public_key.pem";

  return Keys.Ed25519.parseKeyFiles(publicKeyPath, privateKeyPath);
};

const FUND_AMOUNT = Number(process.env.FUND_AMOUNT) || 100000000000000;
const PAYMENT_AMOUNT = Number(process.env.PAYMENT_AMOUNT) || 1000000000000;

const NODE_URL = process.env.NODE_URL || "http://localhost:40101/rpc";
const WASM_PATH = process.env.WASM_PATH;
const NETWORK_NAME = process.env.NETWORK_NAME || "casper-net-1";
const BASE_KEY_PATH = process.env.BASE_KEY_PATH;

// Get a faucet account from provided path
export const faucetAccount = getAccountFromKeyPair(BASE_KEY_PATH!);

// Create a client connected to Casper Node
const client = new CasperClient(NODE_URL);

//
// Helper methods
//

async function getDeploy(deployHash: string) {
  while (true) {
    const [deploy, raw] = await client.getDeploy(deployHash);
    if (raw.execution_results.length !== 0) {
      console.log(raw.execution_results);
      if (raw.execution_results[0].result.Success) {
        return deploy;
      } else {
        console.log(
          JSON.stringify(raw.execution_results[0].result.Failure, null, 2),
        );
        throw Error(
          "Contract execution: " +
            raw.execution_results[0].result.Failure?.error_message,
        );
      }
    } else {
      await sleep(1000);
    }
  }
}

export async function getBlockState(contractHash: string) {
  const c = new CasperServiceByJsonRPC(NODE_URL);
  const blockResult = await c.getLatestBlockInfo();

  const stateRootHash = blockResult.block?.header.state_root_hash;

  const blockState = await c
    .getBlockState(stateRootHash!, `hash-${contractHash}`, [])
    .then(res => res.Contract);

  return blockState;
}

async function getAccount(publicKey: CLPublicKey) {
  const c = new CasperServiceByJsonRPC(NODE_URL);
  const blockResult = await c.getLatestBlockInfo();

  const stateRootHash = blockResult.block?.header.state_root_hash;

  const account = await c
    .getBlockState(stateRootHash!, publicKey.toAccountHashStr(), [])
    .then(res => res.Account);

  return account;
}

async function sendDeploy(deploy: Deploy, signingKeys: AsymmetricKey[]) {
  for (let key of signingKeys) {
    console.log(`Signed by: ${key.publicKey.toAccountHashStr()}`);
    deploy = client.signDeploy(deploy, key);
  }
  const deployHash = await client.putDeploy(deploy);
  await printDeploy(deployHash);
}

// Helper method to create a new hierarchical deterministic wallet
function randomMasterKey() {
  const seed = new Uint8Array(randomSeed());
  return client.newHdWallet(seed);
}

function makeMasterKey(seed: Uint8Array) {
  return client.newHdWallet(seed);
}

// Helper method for printing deploy result
async function printDeploy(deployHash: string) {
  console.log("Deploy hash: " + deployHash);
  console.log("Deploy result:");

  const deployRes = await getDeploy(deployHash);

  console.log(JSON.stringify(DeployUtil.deployToJson(deployRes), null, 2));
}

// Helper method for printing account info
async function printAccount(account: Secp256K1) {
  console.log("\n[x] Current state of the account:");
  const accountRes = await getAccount(account.publicKey);
  console.log(JSON.stringify(accountRes, null, 2));
  return accountRes;
}

//
// Transfers
//

// Builds native transfer deploy
function transferDeploy(
  fromAccount: Secp256K1,
  toAccount: CLPublicKey,
  amount: number,
) {
  const deployParams = new DeployUtil.DeployParams(
    fromAccount.publicKey,
    NETWORK_NAME,
  );
  const transferParams = DeployUtil.ExecutableDeployItem.newTransfer(
    amount,
    toAccount,
    null,
    1,
  );
  const payment = DeployUtil.standardPayment(PAYMENT_AMOUNT);
  return DeployUtil.makeDeploy(deployParams, transferParams, payment);
}

// Helper method for funding the specified account from a faucetAccount
async function fundAccount(account: CLPublicKey) {
  const deploy = transferDeploy(faucetAccount, account, FUND_AMOUNT);
  await sendDeploy(deploy, [faucetAccount]);
}

//
// Contract deploy related methods
//

function buildContractInstallDeploy(
  baseAccount: Secp256K1,
  contractName: string,
  args: Record<string, CLValue>,
) {
  const deployParams = new DeployUtil.DeployParams(
    baseAccount.publicKey,
    NETWORK_NAME,
  );

  const session = new Uint8Array(
    fs.readFileSync(WASM_PATH! + contractName, null).buffer,
  );

  const runtimeArgs = RuntimeArgs.fromMap(args);
  const sessionModule = DeployUtil.ExecutableDeployItem.newModuleBytes(
    session,
    runtimeArgs,
  );
  const payment = DeployUtil.standardPayment(PAYMENT_AMOUNT);

  return DeployUtil.makeDeploy(deployParams, sessionModule, payment);
}

function buildDeploy(
  baseAccount: Secp256K1,
  contractHash: Uint8Array,
  entrypoint: string,
  args: Record<string, CLValue>,
) {
  const deployParams = new DeployUtil.DeployParams(
    baseAccount.publicKey,
    NETWORK_NAME,
  );
  const runtimeArgs = RuntimeArgs.fromMap(args);
  const sessionModule = DeployUtil.ExecutableDeployItem.newStoredContractByHash(
    contractHash,
    entrypoint,
    runtimeArgs,
  );
  const payment = DeployUtil.standardPayment(PAYMENT_AMOUNT);
  return DeployUtil.makeDeploy(deployParams, sessionModule, payment);
}

export const getTotalSupply = async (contractHash: string) => {
  const c = new CasperServiceByJsonRPC(NODE_URL);
  const blockResult = await c.getLatestBlockInfo();

  const stateRootHash = blockResult.block?.header.state_root_hash;

  const value = await c
    .getBlockState(stateRootHash!, contractHash, ["total_supply"])
    .then(res => res.CLValue as CLU256);

  return value;
};

export const getDeployInfo = async (deployHash: string) => {
  const c = new CasperServiceByJsonRPC(NODE_URL);
  const blockResult = await c.getLatestBlockInfo();

  return c.getDeployInfo(deployHash);
};

export const getBlockTransfers = async (blockHash: string) => {
  const c = new CasperServiceByJsonRPC(NODE_URL);
  const blockResult = await c.getLatestBlockInfo();

  const stateRootHash = blockResult.block?.header.state_root_hash;

  const value = await c.getBlockTransfers(blockHash);

  return value;
};

export const getLatestBlockInfo = async () => {
  const c = new CasperServiceByJsonRPC(NODE_URL);
  const blockResult = await c.getLatestBlockInfo();

  return blockResult;
};

export const getInfo = async () => {
  const c = new CasperServiceByJsonRPC(NODE_URL);
  // const c = new CasperServiceByJsonRPC("http://65.108.78.120:7777/rpc");
  const blockResult = await c.getStatus();

  return blockResult;
};

export function approveBurnProof(
  fromAccount: Secp256K1,
  contractHash: Uint8Array,

  //
  proofHash: CLU256,
) {
  return buildDeploy(fromAccount, contractHash, "approve_burn_proof", {
    proof_hash: proofHash,
  });
}

export function setAllowance(
  fromAccount: Secp256K1,
  contractHash: Uint8Array,

  //
  args: {
    mint_token: CLList<CLU8>;
    burn_token: CLList<CLU8>;
    mint_chain_type: CLU8;
    mint_chain_id: CLU32;
    burn_chain_type: CLU8;
    burn_chain_id: CLU32;
  },
) {
  return buildDeploy(fromAccount, contractHash, "set_allowance", args);
}

export function mintWithBurnProof(
  fromAccount: Secp256K1,
  contractHash: Uint8Array,
  //
  args: {
    mint_token: CLByteArray;
    burn_token: CLList<CLU8>;
    burn_caller: CLList<CLU8>;
    burn_chain_type: CLU8;
    burn_chain_id: CLU32;
    burn_amount: CLU256;
    burn_proof_hash: CLU256;
    burn_nonce: CLU256;
  },
) {
  return buildDeploy(fromAccount, contractHash, "mint_with_burn_proof", args);
}

export function burnAndCreateProof(
  fromAccount: Secp256K1,
  contractHash: Uint8Array,

  //
  args: {
    burn_token: CLByteArray;
    mint_token: CLList<CLU8>;
    mint_caller: CLList<CLU8>;
    mint_chain_type: CLU8;
    mint_chain_id: CLU32;
    burn_amount: CLU256;
  },
) {
  return buildDeploy(fromAccount, contractHash, "burn_and_create_proof", args);
}

function __tester(
  fromAccount: Secp256K1,
  contractHash: Uint8Array,

  //
  destToken: CLList<CLU8>,
  destCaller: CLList<CLU8>,
  srcAmount: CLU256,
  destChainType: CLU8,
  destChainId: CLU32,
) {
  return buildDeploy(fromAccount, contractHash, "tester", {
    dest_token: destToken,
    dest_caller: destCaller,
    src_amount: srcAmount,
    dest_chain_type: destChainType,
    dest_chain_id: destChainId,
  });
}

export function mint(
  fromAccount: Secp256K1,
  contractHash: Uint8Array,
  amount: CLU256,
  to: CLKey,
) {
  return buildDeploy(fromAccount, contractHash, "mint", {
    amount,
    to,
  });
}

export {
  randomMasterKey,
  makeMasterKey,
  fundAccount,
  printAccount,
  sendDeploy,
  buildContractInstallDeploy,
};

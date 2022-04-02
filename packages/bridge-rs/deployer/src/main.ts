require("dotenv").config();

import { BigNumber } from "@ethersproject/bignumber";
import {
  CLAccountHash,
  CLByteArray,
  CLPublicKey,
  CLValueBuilder,
} from "casper-js-sdk";
import { StoredValue } from "casper-js-sdk/dist/lib/StoredValue";
import {
  approveBurnProof,
  buildContractInstallDeploy,
  burnAndCreateProof,
  fundAccount,
  getBlockState,
  getInfo,
  makeMasterKey,
  mint,
  printAccount,
  sendDeploy,
  setAllowance,
} from "./contract";

// prettier-ignore
const superSeed = [21,20,18,48,66,67,64,69,11,38,89,92,82,125,106,9,40,0,26,107,22,75,72,28,62,36,16,1,125,40,84,27,114,17,101,125,2,94,63,113]

const superBridgeContractHash =
  "842274145F6b250e7Be85FBE8435d108D6c5CAc95196cA218971e3B77f6caf17";

const superBridgeContractHashBytes = Buffer.from(
  superBridgeContractHash,
  "hex",
);

const bufferToU8List = (b: Buffer) => {
  return CLValueBuilder.list([...b].map(item => CLValueBuilder.u8(item)));
};

const topUp2 = async () => {
  const masterKey = makeMasterKey(new Uint8Array(superSeed));
  const mainAccount = masterKey.deriveIndex(1);
  await fundAccount(mainAccount.publicKey);
};

const topUp = async () => {
  const account = CLPublicKey.fromHex(
    "01fd26775e8e54349ba20be837844a20f841a17090caf35ec8a8352bf2468a5ed4",
  );

  await fundAccount(account);
};

const setAllowances = async () => {
  await getInfo().then(a => console.log(a));

  const masterKey = makeMasterKey(new Uint8Array(superSeed));
  const mainAccount = masterKey.deriveIndex(1);

  console.log("Main account: " + mainAccount.publicKey.toAccountHashStr());

  const deployerAccount = mainAccount;

  let accountInfo: StoredValue["Account"];

  // ---

  const _20bytes = Array.from({ length: 40 }).fill(0).join("");
  const _8bytes = Array.from({ length: 16 }).fill(0).join("");

  const evmChainType = 1;
  const evmChainId = 1337;
  const evmTokens = [
    "0B0e3eE1E9075E5574Fb7470804F30521f918d21",
    // "fA91e958C297B8E57DaFb8c275dD108A5AbF7C8F",
    // "9bA7D8a6Ba4AAbEfa77aDA0821b95CB3AB73478b",
  ]
    .map(item => Buffer.from(_20bytes + item, "hex"))
    .map(item => bufferToU8List(item));

  const casperTokens = [
    "4a361C1F0D019A5662c7983f362f02524E30E9aF0653f58f60ae1001D872f001", // supCSPR
    // "42061A185056b3e9cB142CC713Abbc7D0EE12c1ffDD0039f14E5EaF42D278AD8", // supETH
    // "DA0d9619631fFdaa4363deB85a90B5e2c25c38eb1a026192f0CB1F33B84126Da", // supUSDC
  ].map(item => bufferToU8List(Buffer.from(_8bytes + item, "hex")));

  // 3 combos
  // set allowance only for those tokens that are similar
  const combos = evmTokens.map(
    (evmToken, i) => [evmToken, casperTokens[i]] as const,
  );

  const deploys = combos.map(([evmToken, casperToken]) => {
    return setAllowance(deployerAccount, superBridgeContractHashBytes, {
      mint_token: casperToken,
      burn_token: evmToken,
      mint_chain_type: CLValueBuilder.u8(2),
      mint_chain_id: CLValueBuilder.u32(1010),
      burn_chain_type: CLValueBuilder.u8(1),
      burn_chain_id: CLValueBuilder.u32(1337),
    });
  });

  const _ = await Promise.all(
    deploys.map(deploy => sendDeploy(deploy, [deployerAccount])),
  );

  accountInfo = await printAccount(deployerAccount);

  console.log(JSON.stringify(accountInfo, null, 2));
};

const deployTokens = async () => {
  const masterKey = makeMasterKey(new Uint8Array(superSeed));
  const mainAccount = masterKey.deriveIndex(1);

  console.log("Main account: " + mainAccount.publicKey.toAccountHashStr());

  // console.log("\n[x] Funding main account.");
  // await fundAccount(mainAccount);
  // await printAccount(mainAccount);

  const deployerAccount = mainAccount;

  let accountInfo: StoredValue["Account"];
  console.log("\n[x] Install contract");

  const tokens = [
    ["sup_cspr_token", "supCSPR", 18, 1e10],
    ["sup_eth_token", "supETH", 18, 1e10],
    ["sup_usdc_token", "supUSDC", 18, 1e10],
  ] as const;

  const deploys = tokens.map(
    ([contract_name, symbol, decimals, initial_supply]) => {
      return buildContractInstallDeploy(
        deployerAccount,
        "erc20-contract.wasm",
        {
          contract_name: CLValueBuilder.string(contract_name),
          name: CLValueBuilder.string(symbol),
          symbol: CLValueBuilder.string(symbol),
          decimals: CLValueBuilder.u8(decimals),
          initial_supply: CLValueBuilder.u256(initial_supply),
        },
      );
    },
  );

  const _ = await Promise.all(
    deploys.map(deploy => sendDeploy(deploy, [deployerAccount])),
  );

  accountInfo = await printAccount(deployerAccount);

  console.log(JSON.stringify(accountInfo, null, 2));
};

const accountInfo = async () => {
  const masterKey = makeMasterKey(new Uint8Array(superSeed));
  const mainAccount = masterKey.deriveIndex(1);

  await printAccount(mainAccount);
};

const deployBridge = async () => {
  await getInfo().then(a => console.log(a));

  const masterKey = makeMasterKey(new Uint8Array(superSeed));
  const mainAccount = masterKey.deriveIndex(1);

  console.log("Main account: " + mainAccount.publicKey.toAccountHashStr());

  const deployerAccount = mainAccount;
  let accountInfo: StoredValue["Account"];
  console.log("\n[x] Install contract");

  let deploy = buildContractInstallDeploy(deployerAccount, "contract.wasm", {});

  await sendDeploy(deploy, [deployerAccount]);
  accountInfo = await printAccount(deployerAccount);
};

const approve = async () => {
  const masterKey = makeMasterKey(new Uint8Array(superSeed));
  const mainAccount = masterKey.deriveIndex(1);

  const deployerAccount = mainAccount;

  let deploy = approveBurnProof(
    deployerAccount,
    superBridgeContractHashBytes,
    CLValueBuilder.u256(
      BigNumber.from(
        "0x93cd0f5cfc0bc26ab269cfaa25e9416146d68f669b432d3aa4fa29286d6ea393",
      ),
    ),
  );

  await sendDeploy(deploy, [deployerAccount]);
};

const mintsAndBurns = async () => {
  const masterKey = makeMasterKey(new Uint8Array(superSeed));
  const mainAccount = masterKey.deriveIndex(1);

  const deployerAccount = mainAccount;

  const _20bytes = Array.from({ length: 40 }).fill(0).join("");

  const mintToken = bufferToU8List(
    Buffer.from(_20bytes + "0B0e3eE1E9075E5574Fb7470804F30521f918d21", "hex"),
  );
  const bytes40 = Array.from({ length: 40 }).map(_ => CLValueBuilder.u8(1));

  const srcToken = new CLByteArray(new Uint8Array());

  const evmChainType = 1;
  const evmChainId = 1337;

  const testBurns = [
    [mintToken, bytes40, 500 + (1e18).toString(), evmChainType, evmChainId],
    [mintToken, bytes40, 600 + (1e18).toString(), evmChainType, evmChainId],
  ] as const;

  const deploys = testBurns.map(
    ([mintToken, mintCaller, burnAmount, mintChainType, mintChainId]) =>
      burnAndCreateProof(deployerAccount, superBridgeContractHashBytes, {
        burn_token: new CLByteArray(
          new Uint8Array(
            Buffer.from(
              "4a361C1F0D019A5662c7983f362f02524E30E9aF0653f58f60ae1001D872f001",
              "hex",
            ),
          ),
        ),
        mint_token: mintToken,
        mint_caller: CLValueBuilder.list(mintCaller),
        mint_chain_type: CLValueBuilder.u8(mintChainType),
        mint_chain_id: CLValueBuilder.u32(mintChainId),
        burn_amount: CLValueBuilder.u256(burnAmount),
      }),
  );

  const _ = await Promise.all(
    deploys.map(deploy => sendDeploy(deploy, [deployerAccount])),
  );
};

const mintForAccount = async () => {
  const masterKey = makeMasterKey(new Uint8Array(superSeed));
  const mainAccount = masterKey.deriveIndex(1);

  const deployerAccount = mainAccount;

  // const account = CLPublicKey.fromHex(
  //   "010e0869f62752bfe1e6c918d5d1fff112897efc8259c7da4af4c1aa7586e4ea50",
  // );

  const account = mainAccount.publicKey;

  const contractHash = Buffer.from(
    "4a361C1F0D019A5662c7983f362f02524E30E9aF0653f58f60ae1001D872f001",
    "hex",
  );

  // CLValueBuilder.

  const deploy = mint(
    deployerAccount,
    contractHash,
    CLValueBuilder.u256("1111111" + 1e18),
    CLValueBuilder.key(new CLAccountHash(account.toAccountHash())),
  );

  await sendDeploy(deploy, [deployerAccount]);

  await printAccount(deployerAccount);

  console.log(JSON.stringify(accountInfo, null, 2));
};

const getToken = async () => {
  // const tokenInfo = await getBlockState(
  //   "4a361C1F0D019A5662c7983f362f02524E30E9aF0653f58f60ae1001D872f001",
  // );
  const tokenInfo = await getBlockState(
    "a78925aB37cD56c537446cAD7B3610D900F1c02e1a294c61e7Cf1aa51eC447ee",
  );

  console.log(JSON.stringify(tokenInfo, null, 2));
};

const printAddress = async () => {
  const account = CLPublicKey.fromHex(
    "010e0869f62752bfe1e6c918d5d1fff112897efc8259c7da4af4c1aa7586e4ea50",
  );

  console.log(account.toAccountHashStr());
  console.log(account.toHex());

  return;
  const masterKey = makeMasterKey(new Uint8Array(superSeed));
  const mainAccount = masterKey.deriveIndex(1);

  await printAccount(mainAccount);

  console.log(mainAccount.publicKey.toAccountHashStr());
  console.log(mainAccount.publicKey.toHex());

  // console.log(mainAccount.publicKey.toAccountHashStr());
  // console.log(mainAccount.publicKey.toAccountHash());
};

// fire();
// deployBridge();
// topUp2();
// deployTokens();
// mintsAndBurns();
// mintForAccount();
// setAllowances();
// topUp2();
// deployBridge();
// topUp();
// mintForAccount();
// getToken();
// printAddress();
// mintForAccount();
accountInfo();

// deployTokens();
// getToken();
// getInfo().then(a => console.log(a));

// approve();
// topUp();

// getToken();

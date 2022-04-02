const entryPoints = [
  "burn_and_create_proof",
  "mint_with_burn_proof",
  "approve_burn_proof",
] as const;

type EntryPoint = typeof entryPoints[number];

export interface SuperJsonBlock {
  hash: string; //JsonBlockHash;
  header: JsonHeader;
  proofs: string[];
  body: {
    proposer: string;
    deploy_hashes: string[];
    transfer_hashes: string[];
  };
}

interface StoredContractByHash {
  hash: string;
  StoredContractByHash: {
    entry_point: EntryPoint;
  };
  args: any[]; // ignore
}

type Transform =
  | "Identity"
  | "AddUInt512"
  | {
      WriteCLValue: {
        cl_type:
          | "U512"
          | "Any"
          | {
              List: "U8";
            };
        bytes: string;
        parsed: any | null;
      };
    }
  | {
      WriteDeployInfo: any;
    };

type TransformKey = {
  key: string;
  transform: Transform;
};

interface ExecutionResultSuccess {
  effect: {
    operations: any[]; // TODO:
    transforms: TransformKey[];
  };
  transfers: any[]; // TODO:
  cost: string;
}

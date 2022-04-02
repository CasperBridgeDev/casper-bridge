use crate::shared::encode_hex;
use std::{path::PathBuf, rc::Rc};

use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG,
    DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_execution_engine::{
    core::engine_state::{run_genesis_request::RunGenesisRequest, ExecutionResult, GenesisAccount},
    storage::global_state::in_memory::InMemoryGlobalState,
};
use casper_types::{
    account::AccountHash, bytesrepr::Bytes, runtime_args, ContractHash, ContractPackage, Key,
    Motes, PublicKey, RuntimeArgs, SecretKey, URef, U256, U512,
};

pub struct TestFixture {
    builder: WasmTestBuilder<InMemoryGlobalState>,
    account: AccountHash,
    account_2: AccountHash,
}

const MY_ACCOUNT: [u8; 32] = [7u8; 32];
const MY_ACCOUNT_2: [u8; 32] = [6u8; 32];
const CONTRACT_WASM: &str = "contract.wasm";
const TOKEN_CONTRACT_WASM: &str = "erc20-contract.wasm";

impl TestFixture {
    pub fn deploy() -> Self {
        let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_addr = AccountHash::from(&public_key);

        let account = GenesisAccount::account(
            public_key,
            Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
            None,
        );

        let secret_key_2 = SecretKey::ed25519_from_bytes(MY_ACCOUNT_2).unwrap();
        let public_key_2 = PublicKey::from(&secret_key_2);
        let account_addr_2 = AccountHash::from(&public_key_2);
        let account_2 = GenesisAccount::account(
            public_key_2,
            Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
            None,
        );

        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        genesis_config.ee_config_mut().push_account(account);
        genesis_config.ee_config_mut().push_account(account_2);

        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
        );

        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&run_genesis_request).commit();

        let execute_request = ExecuteRequestBuilder::standard(
            account_addr,
            TOKEN_CONTRACT_WASM,
            runtime_args! {
                "contract_name" => "super_token".to_string(),
                "name" => "SUPER".to_string(),
                "symbol" => "SUPER".to_string(),
                "decimals" => 18 as u8,
                "initial_supply" => U256::from(0u64)
            },
        )
        .build();

        // // deploy the contract.
        builder.exec(execute_request).commit().expect_success();

        // let token_contract_hash = wrapped.token_contract_hash();

        let execute_request =
            ExecuteRequestBuilder::standard(account_addr, CONTRACT_WASM, runtime_args! {}).build();

        // deploy the contract.
        builder.exec(execute_request).commit().expect_success();

        Self {
            account: account_addr,
            account_2: account_addr_2,
            builder: builder,
        }
    }

    pub fn mint(&mut self, amount: U256, to: AccountHash) {
        let execute_request = ExecuteRequestBuilder::contract_call_by_hash(
            self.account,
            self.token_contract_hash().into(),
            "mint",
            runtime_args! {
                "amount" => amount,
                "to" => Key::from(to),
            },
        )
        .build();

        let tx = self.builder.exec(execute_request).commit();
        tx.expect_success();
    }

    pub fn get_burn_proof_storage_uref(&self) -> URef {
        let some = self.builder.query(
            None,
            Key::Account(self.account),
            &["bridge_contract_hash".to_string()],
        );

        let some = some.unwrap();

        let some = some.as_contract().unwrap().named_keys();
        let uref = match some.get("burn_proof_storage").unwrap() {
            Key::URef(uref) => uref,
            _ => panic!(),
        };

        *uref
    }

    pub fn token_balances_uref(&self) -> URef {
        let some = self.builder.query(
            None,
            Key::Account(self.account),
            &["super_token_contract_hash".to_string()],
        );

        let some = some.unwrap();

        let some = some.as_contract().unwrap().named_keys();
        let uref = match some.get("balances").unwrap() {
            Key::URef(uref) => uref,
            _ => panic!(),
        };

        *uref
    }

    pub fn bridge_nonces_uref(&self) -> URef {
        let some = self.builder.query(
            None,
            Key::Account(self.account),
            &["bridge_contract_hash".to_string()],
        );

        let some = some.unwrap();

        let some = some.as_contract().unwrap().named_keys();
        let uref = match some.get("nonces").unwrap() {
            Key::URef(uref) => uref,
            _ => panic!(),
        };

        *uref
    }

    pub fn token_contract_hash(&self) -> ContractHash {
        let token_contract_hash = self
            .builder
            .query(
                None,
                Key::Account(self.account),
                &["super_token_contract_hash_wrapped".to_string()],
            )
            .unwrap()
            .as_cl_value()
            .unwrap()
            .clone()
            .into_t::<ContractHash>()
            .expect("some");

        token_contract_hash
    }

    pub fn set_allowance(
        &mut self,
        mint_token: Bytes,
        burn_token: Bytes,
        mint_chain_type: u8,
        mint_chain_id: u32,
        burn_chain_type: u8,
        burn_chain_id: u32,

        is_ok: bool,
    ) {
        let execute_request = ExecuteRequestBuilder::contract_call_by_hash(
            self.account,
            self.contract_hash().into(),
            "set_allowance",
            runtime_args! {
                "mint_token" =>  mint_token,
                "burn_token" =>  burn_token,
                "mint_chain_type" => mint_chain_type,
                "mint_chain_id" => mint_chain_id,
                "burn_chain_type" => burn_chain_type,
                "burn_chain_id" => burn_chain_id,
            },
        )
        .build();

        let tx = self.builder.exec(execute_request).commit();

        if is_ok {
            tx.expect_success();
        } else {
            tx.expect_failure();
        }
    }

    pub fn mint_with_burn_proof(
        &mut self,
        mint_token: ContractHash,
        burn_token: Bytes,
        burn_caller: Bytes,
        burn_chain_type: u8,
        burn_chain_id: u32,
        burn_proof_hash: U256,
        burn_amount: U256,
        burn_nonce: U256,
        is_ok: bool,
    ) {
        let execute_request = ExecuteRequestBuilder::contract_call_by_hash(
            self.account,
            self.contract_hash().into(),
            "mint_with_burn_proof",
            runtime_args! {
                "mint_token" => mint_token,
                "burn_token" => burn_token,
                "burn_caller" => burn_caller,
                "burn_chain_type" => burn_chain_type,
                "burn_chain_id" => burn_chain_id,
                "burn_amount" => burn_amount,
                "burn_proof_hash" => burn_proof_hash,
                "burn_nonce" => burn_nonce,
            },
        )
        .build();

        let tx = self.builder.exec(execute_request).commit();

        if is_ok {
            tx.expect_success();
        } else {
            tx.expect_failure();
        }
    }

    pub fn burn_and_create_proof(
        &mut self,
        burn_token: ContractHash,
        mint_token: Bytes,
        mint_caller: Bytes,
        mint_chain_type: u8,
        mint_chain_id: u32,
        burn_amount: U256,

        is_ok: bool,
    ) {
        let execute_request = ExecuteRequestBuilder::contract_call_by_hash(
            self.account,
            self.contract_hash().into(),
            "burn_and_create_proof",
            runtime_args! {
                "burn_token" => burn_token,
                "mint_token" => mint_token,
                "mint_caller" => mint_caller,
                "mint_chain_type" => mint_chain_type,
                "mint_chain_id" => mint_chain_id,
                "burn_amount" => burn_amount,
            },
        )
        .build();

        let tx = self.builder.exec(execute_request).commit();

        if is_ok {
            tx.expect_success();
        } else {
            tx.expect_failure();
        }
    }

    pub fn approve_burn_proof(&mut self, proof_hash: U256, is_ok: bool) {
        let execute_request = ExecuteRequestBuilder::contract_call_by_hash(
            self.account,
            self.contract_hash().into(),
            "approve_burn_proof",
            runtime_args! {
              "proof_hash" => proof_hash,
            },
        )
        .build();

        let tx = self.builder.exec(execute_request).commit();

        if is_ok {
            tx.expect_success();
        } else {
            tx.expect_failure();
        }
    }

    pub fn account(&self) -> AccountHash {
        self.account
    }

    pub fn get_balance(&self, account: &str) -> U256 {
        let balances_uref = self.token_balances_uref();

        let balance = self
            .builder
            .query_dictionary_item(None, balances_uref, account);

        // account not found
        if balance.is_err() {
            return 0.into();
        }

        let balance = balance
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t::<Option<U256>>()
            .expect("should be");

        balance.unwrap()
    }

    pub fn get_nonce_by_token(&self, token_hash: ContractHash) -> U256 {
        let nonces_uref = self.bridge_nonces_uref();
        let token_hex = encode_hex(&token_hash.as_bytes().to_vec());

        let nonce = self
            .builder
            .query_dictionary_item(None, nonces_uref, &token_hex);

        // token not found
        if nonce.is_err() {
            return 0.into();
        }

        let nonce = nonce
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t::<Option<U256>>()
            .expect("should be");

        nonce.unwrap()
    }

    pub fn get_burn_proof_status(&self, tx_hash: &str) -> Option<u8> {
        let burn_proof_storage_uref = self.get_burn_proof_storage_uref();

        let tx_hash = self
            .builder
            .query_dictionary_item(None, burn_proof_storage_uref, tx_hash);

        // tx hash not found
        if tx_hash.is_err() {
            return None;
        }

        let tx_hash = tx_hash
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t::<Option<u8>>()
            .expect("should be");

        tx_hash
    }

    pub fn total_supply(&self) -> U256 {
        self.builder
            .query(
                None,
                self.token_contract_hash().into(),
                &["total_supply".to_string()],
            )
            .expect("should be stored value")
            .as_cl_value()
            .expect("should be cl value")
            .clone()
            .into_t::<U256>()
            .expect("should be U256")
    }

    pub fn contract_hash(&self) -> ContractHash {
        let some = self
            .builder
            .query(
                None,
                Key::Account(self.account),
                &["bridge_package_hash".to_string()],
            )
            .expect("should be stored value")
            .as_contract_package()
            .expect("should be cl value")
            .current_contract_hash()
            .expect("should be contract hash");

        some
    }
}

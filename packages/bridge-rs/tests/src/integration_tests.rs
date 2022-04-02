#[cfg(test)]
mod test_fixture;

#[cfg(test)]
mod shared;

#[cfg(test)]
mod tests {
    use crate::shared::pad_with_8_bytes;
    use std::{path::PathBuf, str::FromStr};

    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG,
        DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
    };
    use casper_execution_engine::core::engine_state::{
        run_genesis_request::RunGenesisRequest, GenesisAccount,
    };
    use casper_types::{
        account::AccountHash,
        bytesrepr::{Bytes, FromBytes, ToBytes},
        runtime_args, Key, Motes, PublicKey, RuntimeArgs, SecretKey, URef, U256, U512,
    };

    use crate::{
        shared::{encode_hex, merge_bytes, sha256, u256_to_bytes, u256_to_hex},
        test_fixture::TestFixture,
    };

    #[derive(Debug, PartialEq, Eq)]

    enum States {
        Burned = 1,
        Approved = 2,
        Executed = 3,
    }

    impl From<u8> for States {
        fn from(val: u8) -> Self {
            match val {
                1 => States::Burned,
                2 => States::Approved,
                3 => States::Executed,
                _ => panic!(0),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum ChainType {
        Undefined = 0,
        Evm = 1,
        Casper = 2,
        Solana = 3,
        Radix = 4,
    }

    // TODO: update tests
    // TODO: check against errors from contract
    // TODO: check that vasya can't mint petya's hash
    #[test]

    fn tester() {
        let mut deployed = TestFixture::deploy();

        let burn_proof_storage_uref = deployed.get_burn_proof_storage_uref();
        let token = deployed.token_contract_hash();

        {
            {
                let mint_chain_type = ChainType::Casper as u8;
                let mint_chain_id = 1010 as u32;
                let mint_token = Bytes::from(pad_with_8_bytes(
                    deployed.token_contract_hash().as_bytes().to_vec(),
                ));

                let burn_chain_type = ChainType::Evm as u8;
                let burn_chain_id = 1337 as u32;
                let burn_token = Bytes::from(vec![2; 40]);

                deployed.set_allowance(
                    mint_token.clone(),
                    burn_token.clone(),
                    mint_chain_type,
                    mint_chain_id,
                    burn_chain_type,
                    burn_chain_id,
                    true,
                );
            }
            // {
            //     let mint_chain_type = ChainType::Evm as u8;
            //     let mint_chain_id = 42 as u32;
            //     let mint_token = Bytes::from(vec![5; 40]);
            //     deployed.set_allowance(token, mint_token, mint_chain_type, mint_chain_id, true);
            // }
            // {
            //     let mint_chain_type = ChainType::Evm as u8;
            //     let mint_chain_id = 42 as u32;
            //     let mint_token = Bytes::from(vec![2; 40]);
            //     deployed.set_allowance(token, mint_token, mint_chain_type, mint_chain_id, true);
            // }
        }

        println!("burn_proof_storage_uref {:#?}", burn_proof_storage_uref);

        let account = deployed.account();
        println!("account {:#?}", account.to_formatted_string());
        println!("account {:#?}", account.to_string());

        {
            let total_supply = deployed.total_supply();
            assert!(total_supply == 0.into());
        }

        // direct mint for initial supply
        let _ = deployed.mint(U256::from(1_000_000), account);
        println!("minted!");

        // owner
        {
            let account_balance = deployed.get_balance(&account.to_string());
            assert!(account_balance == 1_000_000.into());
            println!("account_balance {:#?}", account_balance);
        }

        // account not found
        {
            let account = "2211000000000000000000000000000000000000000000000000000000000000";

            let account_balance = deployed.get_balance(&account.to_string());
            assert!(account_balance == 0.into());

            println!("account_balance {:#?}", account_balance);
        }

        // let token_contract_hash = deployed.token_contract_hash();

        // tx hash with arbitary amount
        // imagine it was sent from validator (from evm network)
        let burn_proof = {
            let mint_caller_bytes = pad_with_8_bytes(account.as_bytes().to_vec());
            let burn_caller_bytes = vec![1; 40]; // arbitary address

            let mint_token_bytes =
                pad_with_8_bytes(deployed.token_contract_hash().as_bytes().to_vec());
            let burn_token_bytes = vec![2; 40]; // arbitary token

            let burn_amount_bytes = u256_to_bytes(&50_555.into());

            let mint_chain_type_bytes = (ChainType::Casper as u8).to_be_bytes().to_vec();
            let mint_chain_id_bytes = (1010 as u32).to_be_bytes().to_vec(); // 1010 as test chain id for casper

            let burn_chain_type_bytes = (ChainType::Evm as u8).to_be_bytes().to_vec();
            let burn_chain_id_bytes = (1337 as u32).to_be_bytes().to_vec(); // kovan

            let burn_nonce_bytes = u256_to_bytes(&1337.into());

            #[rustfmt::skip]
            let data = merge_bytes(vec![
                mint_caller_bytes, burn_caller_bytes,
                mint_token_bytes, burn_token_bytes,
                burn_amount_bytes,
                mint_chain_type_bytes, mint_chain_id_bytes,
                burn_chain_type_bytes, burn_chain_id_bytes,
                burn_nonce_bytes
            ]);

            let burn_proof = sha256(&data);

            burn_proof
        };

        // approve specific hash
        let _ = deployed.approve_burn_proof(U256::from_big_endian(&burn_proof), true);

        {
            let str_tx_hash = encode_hex(&burn_proof);
            let some = deployed
                .get_burn_proof_status(&str_tx_hash)
                .map(States::from);

            assert!(some == Some(States::Approved));
        }

        println!("transaction from validator approved!");

        let mut minter_fn = |is_ok: bool| {
            deployed.mint_with_burn_proof(
                token,
                Bytes::from(vec![2; 40]),
                Bytes::from(vec![1; 40]),
                ChainType::Evm as u8,
                1337,
                U256::from_big_endian(&burn_proof),
                50_555.into(),
                1337.into(),
                is_ok,
            )
        };

        minter_fn(true); // mint once
        minter_fn(false); // cant mint twice

        println!("transaction from validator minted once!");

        // owner
        {
            let account_balance = deployed.get_balance(&account.to_string());
            assert!(account_balance == 1_050_555.into());
            println!("account_balance {:#?}", account_balance);
        }

        // amount exceeded
        {
            let burn_token = token;
            let mint_token = Bytes::from(vec![2; 40]);
            let mint_caller = Bytes::from(vec![7; 40]);
            let burn_amount = U256::from(2_000_001);
            let mint_chain_type = ChainType::Evm as u8;
            let mint_chain_id = 1337 as u32;

            let _ = deployed.burn_and_create_proof(
                burn_token,
                mint_token,
                mint_caller,
                mint_chain_type,
                mint_chain_id,
                burn_amount,
                false,
            );
        }

        // ok
        {
            let burn_token = token;
            let mint_token = Bytes::from(vec![2; 40]);
            let mint_caller = Bytes::from(vec![7; 40]);
            let burn_amount = U256::from(1000);
            let mint_chain_type = ChainType::Evm as u8;
            let mint_chain_id = 1337 as u32;

            let _ = deployed.burn_and_create_proof(
                burn_token,
                mint_token,
                mint_caller,
                mint_chain_type,
                mint_chain_id,
                burn_amount,
                true,
            );
        }
        println!("proof created!");

        // owner
        {
            let account_balance = deployed.get_balance(&account.to_string());
            assert!(account_balance == (1_000_000 + 50_555 - 1_000).into());
        }

        {
            // check nonce and supply again
            let nonce = deployed.get_nonce_by_token(token);
            let total_supply = deployed.total_supply();

            assert!(nonce == 1.into());
            assert!(total_supply == (1_000_000 + 50_555 - 1_000).into());
        }

        // calculate hash
        let burn_proof = {
            let mint_caller_bytes = vec![7; 40];
            let burn_caller_bytes = pad_with_8_bytes(account.as_bytes().to_vec());

            let mint_token_bytes = vec![2; 40];

            let burn_token_bytes =
                pad_with_8_bytes(deployed.token_contract_hash().as_bytes().to_vec());

            let burn_amount_bytes = u256_to_bytes(&1_000.into());

            let mint_chain_type_bytes = (ChainType::Evm as u8).to_be_bytes().to_vec();
            let mint_chain_id_bytes = (1337 as u32).to_be_bytes().to_vec(); // 1010 as test chain id for casper

            let burn_chain_type_bytes = (ChainType::Casper as u8).to_be_bytes().to_vec();
            let burn_chain_id_bytes = (1010 as u32).to_be_bytes().to_vec(); // kovan

            let burn_nonce_bytes = u256_to_bytes(&0.into());

            #[rustfmt::skip]
            let data = merge_bytes(vec![
                mint_caller_bytes, burn_caller_bytes,
                mint_token_bytes, burn_token_bytes,
                burn_amount_bytes,
                mint_chain_type_bytes, mint_chain_id_bytes,
                burn_chain_type_bytes, burn_chain_id_bytes,
                burn_nonce_bytes
            ]);

            let burn_proof = sha256(&data);

            burn_proof
        };

        // should be burned
        {
            let str_tx_hash = encode_hex(&burn_proof);
            let some: Option<States> = deployed
                .get_burn_proof_status(&str_tx_hash)
                .map(States::from);

            assert!(some == Some(States::Burned));
        }

        // cant approve because it already exists on chain
        let _ = deployed.approve_burn_proof(U256::from_big_endian(&burn_proof), false);

        // should not be approved
        // check once again
        {
            let str_tx_hash = encode_hex(&burn_proof);
            let some: Option<States> = deployed
                .get_burn_proof_status(&str_tx_hash)
                .map(States::from);
            assert!(some == Some(States::Burned));
        }

        println!("burned transaction not approved!");

        // should not be minted
        let _ = deployed.mint_with_burn_proof(
            token,
            Bytes::from(vec![2; 40]),
            Bytes::from(vec![1, 40]),
            1,
            1337,
            U256::from_big_endian(&burn_proof),
            1_000.into(),
            0.into(),
            false,
        );

        println!("burned transaction not minted!");

        {
            // check nonce and supply again
            let nonce = deployed.get_nonce_by_token(token);
            let total_supply = deployed.total_supply();

            assert!(nonce == 1.into());
            assert!(total_supply == (1_000_000 + 50_555 - 1_000).into());
        }
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}

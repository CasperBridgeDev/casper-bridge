#![no_std]
#![no_main]
// #![feature(default_alloc_error_handler)]

extern crate alloc;

use casper_types::bytesrepr::Bytes;
use casper_types::contracts::NamedKeys;
use endpoints::endpoint;
// use hex;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use casper_types::{
  account::AccountHash,
  bytesrepr::{FromBytes, ToBytes},
  CLTyped, CLValue, ContractHash, URef, U128, U256,
};

use casper_contract::{
  contract_api::{
    account,
    runtime::{self, revert},
    storage, system,
  },
  unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{ApiError, CLType, EntryPoints, Key, Parameter};
use erc20::{SuperToken, ERC20Trait};
use shared::{
  empty_dict, encode_hex, get_key, merge_bytes, set_key, sha256, u256_to_bytes, u256_to_hex,
  u32_to_hex, u8_to_hex, Dict,
};

mod endpoints;
mod erc20;
mod shared;

#[repr(u16)]
enum Error {
  AlreadyApproved = 0,
  AmountExceeded,        // 1
  NotApprovedOrExecuted, // 2
  ProvidedHashIsInvalid, // 3
  InvalidCallerLength,   // 4
  InvalidTokenLength,    // 5
  InvalidPackage,        // 6
  UnknownState,          // 7
  UnknownChain,          // 8
  UnknownAllowance,      // 9
  AllowanceNotFound,     // 10
  MissingApproverRole,   // 11
}

impl From<Error> for ApiError {
  fn from(error: Error) -> Self {
    ApiError::User(error as u16)
  }
}

trait Thingy {
  fn to_hex(&self) -> String;
}

impl Thingy for Vec<u8> {
  fn to_hex(&self) -> String {
    return encode_hex(&self);
  }
}

impl Thingy for U256 {
  fn to_hex(&self) -> String {
    return u256_to_hex(&self);
  }
}

impl Thingy for u8 {
  fn to_hex(&self) -> String {
    return u8_to_hex(&self);
  }
}

impl Thingy for u32 {
  fn to_hex(&self) -> String {
    return u32_to_hex(&self);
  }
}

// WORK IN PROGRESS

pub enum BridgeEvent {
  ProofOfBurn {
    mint_token: Bytes,
    burn_token: Bytes,
    mint_caller: Bytes,
    burn_caller: Bytes,
    burn_amount: U256,
    burn_nonce: U256,
    mint_chain_type: ChainType,
    mint_chain_id: u32,
    burn_chain_type: ChainType,
    burn_chain_id: u32,
    burn_proof_hash: U256,
  },
  ProofOfMint {
    mint_token: Bytes,
    burn_token: Bytes,
    mint_caller: Bytes,
    burn_caller: Bytes,
    burn_amount: U256,
    // burn_nonce ?
    mint_chain_type: ChainType,
    mint_chain_id: u32,
    burn_chain_type: ChainType,
    burn_chain_id: u32,
    burn_proof_hash: U256,
  },
  ApprovedBurnProof {
    burn_proof_hash: U256,
  },
  // FeeUpdated {},
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
enum States {
  Undefined = 0,
  Burned,   // 1
  Approved, // 2
  Executed, // 3
}

impl From<u8> for States {
  fn from(val: u8) -> Self {
    match val {
      1 => States::Burned,
      2 => States::Approved,
      3 => States::Executed,
      _ => revert(Error::UnknownState),
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ChainType {
  Undefined = 0,
  Evm,    // 1
  Casper, // 2
  Solana, // 3
  Radix,  // 4
}

impl From<u8> for ChainType {
  fn from(val: u8) -> Self {
    match val {
      1 => ChainType::Evm,
      2 => ChainType::Casper,
      3 => ChainType::Solana,
      4 => ChainType::Radix,
      _ => revert(Error::UnknownChain),
    }
  }
}


#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Allowance {
  Undefined = 0,
  Allowed, // 1
  Blocked  // 2
}

impl From<u8> for Allowance {
  fn from(val: u8) -> Self {
    match val {
      1 => Allowance::Allowed,
      2 => Allowance::Blocked,
      _ => revert(Error::UnknownAllowance),
    }
  }
}

const ROLE_APPROVER: &str = "ROLE_APPROVER";

const BURN_PROOF_STORAGE_DICT: &str = "burn_proof_storage";
const ALLOWANCES_DICT: &str = "allowances";
const NONCES_DICT: &str = "nonces";
// 


fn get_burn_proof_state(proof_hash: U256) -> States {
  let dict = Dict::at(BURN_PROOF_STORAGE_DICT);

  return dict.get(&proof_hash.to_hex())
    .map(|v: u8| States::from(v))
    .unwrap_or(States::Undefined)
}

fn set_burn_proof_state(proof_hash: U256, state: States)  {
  let dict = Dict::at(BURN_PROOF_STORAGE_DICT);

  dict.set(&proof_hash.to_hex(), state as u8)
}

fn get_allowance_by_hash(hash: Vec<u8>) -> Allowance  {
  let dict = Dict::at(ALLOWANCES_DICT);

  return dict.get(&hash.to_hex())
    .map(|v: u8| Allowance::from(v))
    .unwrap_or(Allowance::Undefined)
}

fn set_allowance_by_hash(hash: Vec<u8>, allowance: Allowance) {
  let dict = Dict::at(ALLOWANCES_DICT);

  dict.set(&hash.to_hex(), allowance as u8)
}


fn get_nonce_by_token(token: ContractHash) -> U256 {
  let dict = Dict::at(NONCES_DICT);
  let token_hex = token.as_bytes().to_vec().to_hex();
  // if token not found, provide zero nonce
  dict.get(&token_hex).unwrap_or(0.into())
}

fn set_nonce_by_token(token: ContractHash, nonce: U256) {
  let dict = Dict::at(NONCES_DICT);
  let token_hex = token.as_bytes().to_vec().to_hex();

  dict.set(&token_hex, nonce)
}

// TODO: use macro
fn only_role(role: &str) {
  let caller = runtime::get_caller();

  let approver_address: AccountHash = get_key(role).unwrap_or_revert();

  require(caller == approver_address, Error::MissingApproverRole);
}


fn print(s: &str) {
  #[cfg(feature = "casper-contract/test-support")]
  runtime::print(s);
}

// sha256(ProofOfBurn) = c5 e1 9c 70 19c477aefcdafef0a3df24119045c1ed6d916e92bd99388b99ba6216
// sha256(ProofOfMint) = ab ba 24 3b 9bae2dcfb6e971870ca2e3c2a64f98edb65ab4cdd1dcda54a4fbf369
// sha256(ApprovedBurnProof) = a4 39 a6 33 2c4168f32836e9fc3a1c1770bd6503c3718aedc53d66544aa65f0191

const PROOF_OF_BURN_SIG: [u8; 4] = [0xC5, 0xE1, 0x9C, 0x70];
const PROOF_OF_MINT_SIG: [u8; 4] = [0xab, 0xba, 0x24, 0x3b];
const APPROVED_BURN_PROOF_SIG: [u8; 4] = [0xa4, 0x39, 0xa6, 0x33];

#[rustfmt::skip]
pub fn emit(bridge_event: BridgeEvent) {  
  let vec_event = match bridge_event {
    BridgeEvent::ProofOfBurn { mint_token, burn_token, mint_caller, burn_caller, burn_amount, burn_nonce, mint_chain_type, mint_chain_id, burn_chain_type, burn_chain_id, burn_proof_hash } => {
      vec![
        PROOF_OF_BURN_SIG.to_vec(),
        mint_token.to_vec(),
        burn_token.to_vec(),
        mint_caller.to_vec(),
        burn_caller.to_vec(),
        u256_to_bytes(&burn_amount),
        u256_to_bytes(&burn_nonce),
        (mint_chain_type as u8).to_be_bytes().to_vec(),
        mint_chain_id.to_be_bytes().to_vec(),
        (burn_chain_type as u8).to_be_bytes().to_vec(),
        burn_chain_id.to_be_bytes().to_vec(),
        u256_to_bytes(&burn_proof_hash),
      ]
    },
    BridgeEvent::ProofOfMint { mint_token, burn_token, mint_caller, burn_caller, burn_amount, mint_chain_type, mint_chain_id, burn_chain_type, burn_chain_id, burn_proof_hash } => {
      vec![
        PROOF_OF_MINT_SIG.to_vec(),
        mint_token.to_vec(),
        burn_token.to_vec(),
        mint_caller.to_vec(),
        burn_caller.to_vec(),
        u256_to_bytes(&burn_amount),
        (mint_chain_type as u8).to_be_bytes().to_vec(),
        mint_chain_id.to_be_bytes().to_vec(),
        (burn_chain_type as u8).to_be_bytes().to_vec(),
        burn_chain_id.to_be_bytes().to_vec(),
        u256_to_bytes(&burn_proof_hash)
      ]
    },
    BridgeEvent::ApprovedBurnProof { burn_proof_hash } => {
      vec![
        APPROVED_BURN_PROOF_SIG.to_vec(),
        u256_to_bytes(&burn_proof_hash)
      ]
    },
  };

  let bytes = merge_bytes(vec_event);
  let _: URef = storage::new_uref(bytes);
}

fn require<T: Into<ApiError>>(is_true: bool, error: T) {
  if !is_true {
    revert(error);
  }
}

// TODO: change on main net to chain id = 1, or any other
const SOURCE_CHAIN_ID: u32 = 1010; // 1010 as test chain id for casper
const SOURCE_CHAIN_TYPE: ChainType = ChainType::Casper;

// TODO: can't bridge to itself
// TODO: do not allow to approve hash on chain it was burned (add fee)
#[no_mangle]
pub fn approve_burn_proof() {
  // guards
  only_role(ROLE_APPROVER);
  // 

  let proof_hash = runtime::get_named_arg::<U256>("proof_hash");

  let some_burn_proof = get_burn_proof_state(proof_hash);

  require(some_burn_proof == States::Undefined, Error::AlreadyApproved);

  set_burn_proof_state(proof_hash, States::Approved);

  emit(BridgeEvent::ApprovedBurnProof {
    burn_proof_hash: proof_hash,
  })
}

fn get_generic_caller() -> Vec<u8> {
  let pad_bytes = vec![0; 8];
  let caller = runtime::get_caller();
  let caller_bytes = caller.to_bytes().unwrap_or_revert();

  require(caller_bytes.len() == 32, Error::InvalidCallerLength);

  // 8 + 32
  merge_bytes(vec![pad_bytes, caller_bytes])
}

fn get_generic_token(token: ContractHash) -> Vec<u8> {
  let pad_bytes = vec![0; 8];
  let token_bytes = token.as_bytes().to_vec();

  require(token_bytes.len() == 32, Error::InvalidTokenLength);

  // 8 + 32
  merge_bytes(vec![pad_bytes, token_bytes])
}

fn get_allowance_hash(
  mint_chain_type: ChainType,
  mint_chain_id: u32,
  burn_chain_type: ChainType,
  burn_chain_id: u32,
  mint_token: Vec<u8>,
  burn_token: Vec<u8>,
) -> Vec<u8> {
  let mint_bytes = {
    let mint_chain_type_bytes = (mint_chain_type as u8).to_be_bytes().to_vec();
    let mint_chain_id_bytes = mint_chain_id.to_be_bytes().to_vec();

    let data = merge_bytes(vec![
      mint_chain_type_bytes,
      mint_chain_id_bytes,
      mint_token,
    ]);

    data
  };

  let burn_bytes = {
    let burn_chain_type_bytes = (burn_chain_type as u8).to_be_bytes().to_vec();
    let burn_chain_id_bytes = burn_chain_id.to_be_bytes().to_vec();

    let data = merge_bytes(vec![
      burn_chain_type_bytes,
      burn_chain_id_bytes,
      burn_token,
    ]);

    data
  };

  if sha256(&mint_bytes) > sha256(&burn_bytes) {
    sha256(&merge_bytes(vec![mint_bytes, burn_bytes]))
  } else {
    sha256(&merge_bytes(vec![burn_bytes, mint_bytes]))
  }
}

// TODO: add fee
// TODO: add bot detection
#[no_mangle]
pub fn mint_with_burn_proof() {
  let mint_token = runtime::get_named_arg::<ContractHash>("mint_token"); // use native address type explicitly

  let burn_token = runtime::get_named_arg::<Bytes>("burn_token");
  let burn_caller = runtime::get_named_arg::<Bytes>("burn_caller");

  let burn_chain_type = runtime::get_named_arg::<u8>("burn_chain_type");
  let burn_chain_id = runtime::get_named_arg::<u32>("burn_chain_id");

  let burn_amount = runtime::get_named_arg::<U256>("burn_amount");

  let burn_proof_hash = runtime::get_named_arg::<U256>("burn_proof_hash");
  let burn_nonce = runtime::get_named_arg::<U256>("burn_nonce");

  require(burn_caller.len() == 40, Error::InvalidCallerLength);
  require(burn_token.len() == 41, Error::InvalidTokenLength);

  let allowance_hash = get_allowance_hash(
    SOURCE_CHAIN_TYPE,
    SOURCE_CHAIN_ID,
    ChainType::from(burn_chain_type),
    burn_chain_id,
    get_generic_token(mint_token),
    burn_token.clone().into()
  );

  require(
    get_allowance_by_hash(allowance_hash) == Allowance::Allowed,
     Error::AllowanceNotFound
  );

  let burn_proof_status = get_burn_proof_state(burn_proof_hash);

  require(
    burn_proof_status == States::Approved,
    Error::NotApprovedOrExecuted,
  );

  let computed_burn_proof_hash = {
    let mint_caller_bytes = get_generic_caller();
    let burn_caller_bytes = burn_caller.to_vec();

    let mint_token_bytes = get_generic_token(mint_token);
    let burn_token_bytes = burn_token.to_vec();

    // burn & mint
    let burn_amount_bytes = u256_to_bytes(&burn_amount);

    let mint_chain_type_bytes = (SOURCE_CHAIN_TYPE as u8).to_be_bytes().to_vec();
    let mint_chain_id_bytes = SOURCE_CHAIN_ID.to_be_bytes().to_vec();

    let burn_chain_type_bytes = burn_chain_type.to_be_bytes().to_vec();
    let burn_chain_id_bytes = burn_chain_id.to_be_bytes().to_vec();

    let burn_nonce_bytes = u256_to_bytes(&burn_nonce);

    #[rustfmt::skip]
    let data = merge_bytes(vec![
      mint_caller_bytes, burn_caller_bytes,
      mint_token_bytes, burn_token_bytes,
      burn_amount_bytes,
      mint_chain_type_bytes, mint_chain_id_bytes,
      burn_chain_type_bytes, burn_chain_id_bytes,
      burn_nonce_bytes
    ]);

    require(
      data.len() ==  234, // 234 = 40 + 40 + 40 + 40 + 32 + 1 + 4 + 1 + 4 + 32
      Error::InvalidPackage,
    );

    sha256(&data)
  };

  let are_hashes_equal = computed_burn_proof_hash == u256_to_bytes(&burn_proof_hash);

  require(are_hashes_equal, Error::ProvidedHashIsInvalid);

  let burn_proof_hash = U256::from_big_endian(&computed_burn_proof_hash);

  set_burn_proof_state(burn_proof_hash, States::Executed);

  let token = SuperToken::new(mint_token);
  let caller = runtime::get_caller();

  token.mint(caller, burn_amount);

  emit(BridgeEvent::ProofOfMint {
    mint_token: get_generic_token(mint_token).into(),
    burn_token,
    mint_caller: get_generic_caller().into(),
    burn_caller,
    burn_amount,
    // burn_nonce ?
    mint_chain_type: SOURCE_CHAIN_TYPE,
    mint_chain_id: SOURCE_CHAIN_ID,
    burn_chain_type: ChainType::from(burn_chain_type),
    burn_chain_id,
    burn_proof_hash
  });
}

#[no_mangle]
pub fn set_allowance() {
  // guards
  only_role(ROLE_APPROVER);
  // 

  let mint_token = runtime::get_named_arg::<Bytes>("mint_token");
  let burn_token = runtime::get_named_arg::<Bytes>("burn_token");

  let mint_chain_type = runtime::get_named_arg::<u8>("mint_chain_type");
  let mint_chain_id = runtime::get_named_arg::<u32>("mint_chain_id");

  let burn_chain_type = runtime::get_named_arg::<u8>("burn_chain_type");
  let burn_chain_id = runtime::get_named_arg::<u32>("burn_chain_id");

  require(mint_token.len() == 40, Error::InvalidTokenLength);
  require(burn_token.len() == 40, Error::InvalidTokenLength);

  let allowance_hash = get_allowance_hash(
    ChainType::from(mint_chain_type),
    mint_chain_id,
    ChainType::from(burn_chain_type),
    burn_chain_id, 
    mint_token.to_vec(),
    burn_token.to_vec(),
  );

  set_allowance_by_hash(allowance_hash, Allowance::Allowed);
}

#[no_mangle]
pub fn burn_and_create_proof() {
  let burn_token = runtime::get_named_arg::<ContractHash>("burn_token"); // use native address type explicitly

  let mint_token = runtime::get_named_arg::<Bytes>("mint_token");
  let mint_caller = runtime::get_named_arg::<Bytes>("mint_caller");

  let mint_chain_type = runtime::get_named_arg::<u8>("mint_chain_type");
  let mint_chain_id = runtime::get_named_arg::<u32>("mint_chain_id");

  let burn_amount = runtime::get_named_arg::<U256>("burn_amount");

  require(mint_caller.len() == 40, Error::InvalidCallerLength);
  require(mint_token.len() == 40, Error::InvalidTokenLength);

  let allowance_hash = get_allowance_hash(
    ChainType::from(mint_chain_type),
    mint_chain_id,
    SOURCE_CHAIN_TYPE,
    SOURCE_CHAIN_ID,
    mint_token.clone().into(),
    get_generic_token(burn_token),
  );

  require(
    get_allowance_by_hash(allowance_hash) == Allowance::Allowed,
     Error::AllowanceNotFound
  );

  let burn_nonce = get_nonce_by_token(burn_token);
  let caller = runtime::get_caller();

  let token = SuperToken::new(burn_token);

  let balance = token.balance_of(caller);

  print(&format!("balance {}", balance));

  require(burn_amount <= balance, Error::AmountExceeded);

  let computed_burn_proof_hash = {
    let mint_caller_bytes = mint_caller.to_vec();
    let burn_caller_bytes = get_generic_caller();

    let mint_token_bytes = mint_token.to_vec();
    let burn_token_bytes = get_generic_token(burn_token);

    // burn & mint
    let burn_amount_bytes = u256_to_bytes(&burn_amount);

    let mint_chain_type_bytes = (mint_chain_type as u8).to_be_bytes().to_vec();
    let mint_chain_id_bytes = mint_chain_id.to_be_bytes().to_vec();

    let burn_chain_type_bytes = (SOURCE_CHAIN_TYPE as u8).to_be_bytes().to_vec();
    let burn_chain_id_bytes =  SOURCE_CHAIN_ID.to_be_bytes().to_vec();

    let burn_nonce_bytes = u256_to_bytes(&burn_nonce);

    #[rustfmt::skip]
    let data = merge_bytes(vec![
      mint_caller_bytes, burn_caller_bytes,
      mint_token_bytes, burn_token_bytes,
      burn_amount_bytes,
      mint_chain_type_bytes, mint_chain_id_bytes,
      burn_chain_type_bytes, burn_chain_id_bytes,
      burn_nonce_bytes
    ]);
    require(
      data.len() ==  234, // 234 = 40 + 40 + 40 + 40 + 32 + 1 + 4 + 1 + 4 + 32
      Error::InvalidPackage,
    );

    sha256(&data)
  };

  print(&format!("burn_proof_hash {}", computed_burn_proof_hash.to_hex()));

  let burn_proof_hash = U256::from_big_endian(&computed_burn_proof_hash);

  set_burn_proof_state(burn_proof_hash, States::Burned);

  token.burn(caller, burn_amount);

  emit(BridgeEvent::ProofOfBurn {
    mint_token,
    burn_token: get_generic_token(burn_token).into(),
    mint_caller,
    burn_caller: get_generic_caller().into(),
    burn_amount,
    burn_nonce,
    mint_chain_type: ChainType::from(mint_chain_type),
    mint_chain_id,
    burn_chain_type: SOURCE_CHAIN_TYPE,
    burn_chain_id: SOURCE_CHAIN_ID,
    burn_proof_hash,
  });

  let next_nonce = burn_nonce + 1;
  set_nonce_by_token(burn_token, next_nonce)

  // TODO: return burn proof hash
  // ret(computed_burn_proof_hash)
}


#[no_mangle]
pub extern "C" fn call() {
  let mut named_keys = NamedKeys::new();

  named_keys.insert(NONCES_DICT.to_string(), empty_dict(NONCES_DICT).into());

  named_keys.insert(
    BURN_PROOF_STORAGE_DICT.to_string(),
    empty_dict(BURN_PROOF_STORAGE_DICT).into(),
  );

  named_keys.insert(
    ALLOWANCES_DICT.to_string(),
  empty_dict(ALLOWANCES_DICT).into(),
  );

  named_keys.insert(
    ROLE_APPROVER.to_string(), 
  storage::new_uref(runtime::get_caller()).into()
  );

  let mut entry_points = EntryPoints::new();
  entry_points.add_entry_point(endpoint(
    "set_allowance",
    vec![
      Parameter::new("mint_token", Bytes::cl_type()),
      Parameter::new("burn_token", Bytes::cl_type()),
      Parameter::new("mint_chain_type", u8::cl_type()),
      Parameter::new("mint_chain_id", u32::cl_type()),
      Parameter::new("burn_chain_type", u8::cl_type()),
      Parameter::new("burn_chain_id", u32::cl_type()),
    ],
    CLType::Unit,
    None,
  ));

  entry_points.add_entry_point(endpoint(
    "burn_and_create_proof",
    vec![
      Parameter::new("burn_token", ContractHash::cl_type()),
      Parameter::new("mint_token", Bytes::cl_type()),
      Parameter::new("mint_caller", Bytes::cl_type()),
      Parameter::new("mint_chain_type", u8::cl_type()),
      Parameter::new("mint_chain_id", u32::cl_type()),
      Parameter::new("burn_amount", U256::cl_type()),
    ],
    CLType::Unit,
    None,
  ));

  entry_points.add_entry_point(endpoint(
    "mint_with_burn_proof",
    vec![
      Parameter::new("mint_token", ContractHash::cl_type()),
      Parameter::new("burn_token", Bytes::cl_type()),
      Parameter::new("burn_caller", Bytes::cl_type()),
      Parameter::new("burn_chain_type", u8::cl_type()),
      Parameter::new("burn_chain_id", u32::cl_type()),
      Parameter::new("burn_amount", U256::cl_type()),
      Parameter::new("burn_proof_hash", U256::cl_type()),
      Parameter::new("burn_nonce", U256::cl_type()),
    ],
    CLType::Unit,
    None,
  ));

  entry_points.add_entry_point(endpoint(
    "approve_burn_proof",
    vec![Parameter::new("proof_hash", U256::cl_type())],
    CLType::Unit,
    None,
  ));

  let (contract_hash, _) = storage::new_contract(
    entry_points,
    Some(named_keys),
    Some("bridge_package_hash".to_string()),
    Some("bridge_access_token".to_string()),
  );

  // TODO: should expose on prod?
  runtime::put_key("bridge_contract_hash", contract_hash.into());
}

use core::convert::TryInto;

use alloc::string::String;
use alloc::vec::Vec;

use core::fmt::Write;
use core::write;

use casper_types::{
  account::AccountHash,
  bytesrepr::{FromBytes, ToBytes},
  CLTyped, CLValue, URef, U256,
};

use casper_contract::{
  contract_api::{
    account, runtime,
    storage::{self, new_dictionary},
    system,
  },
  unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{ApiError, Key};

use sha2::{Digest, Sha256};

pub fn sha256(data: &[u8]) -> Vec<u8> {
  let mut instance = Sha256::new();
  instance.update(data);
  let result = instance.finalize();
  result.to_vec()
}

pub struct Dict {
  uref: URef,
}

impl Dict {
  pub fn at(name: &str) -> Dict {
    let key: Key = runtime::get_key(name).unwrap_or_revert();
    let uref: URef = *key.as_uref().unwrap_or_revert();
    Dict { uref }
  }

  pub fn get<T: CLTyped + FromBytes>(&self, key: &str) -> Option<T> {
    storage::dictionary_get(self.uref, key)
      .unwrap_or_revert()
      .unwrap_or_default()
  }

  pub fn set<T: CLTyped + ToBytes>(&self, key: &str, value: T) {
    storage::dictionary_put(self.uref, key, Some(value));
  }

  pub fn remove<T: CLTyped + ToBytes>(&self, key: &str) {
    storage::dictionary_put(self.uref, key, Option::<T>::None);
  }
}

pub fn get_key<T: FromBytes + CLTyped>(name: &str) -> Option<T> {
  match runtime::get_key(name) {
    None => None,
    Some(value) => {
      let key = value.try_into().unwrap_or_revert();
      let value = storage::read(key).unwrap_or_revert().unwrap_or_revert();
      Some(value)
    }
  }
}

pub fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
  match runtime::get_key(name) {
    Some(key) => {
      let key_ref = key.try_into().unwrap_or_revert();
      storage::write(key_ref, value);
    }
    None => {
      let key = storage::new_uref(value).into();
      runtime::put_key(name, key);
    }
  }
}

pub fn empty_dict(name: &str) -> URef {
  let dict = new_dictionary(name).unwrap_or_revert();

  dict
}

// utils

pub fn merge_bytes(vecs: Vec<Vec<u8>>) -> Vec<u8> {
  let mut data = Vec::new();

  for vec in vecs {
    data.extend(vec);
  }

  data
}

// pub fn decode_hex(s: &str) -> Result<Vec<u8>> {
//   (0..s.len())
//     .step_by(2)
//     .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
//     .collect()
// }

pub fn encode_hex(bytes: &[u8]) -> String {
  let mut s = String::with_capacity(bytes.len() * 2);
  for &b in bytes {
    write!(&mut s, "{:02x}", b).unwrap();
  }
  s
}

// compatible with abi encode (solidity)
pub fn u256_to_bytes(u: &U256) -> Vec<u8> {
  let mut buffer = [0u8; 32];
  u.to_big_endian(&mut buffer);
  buffer.to_vec()
}

pub fn u256_to_hex(u: &U256) -> String {
  let bytes = u256_to_bytes(u);
  encode_hex(&bytes)
}

pub fn u8_to_hex(u: &u8) -> String {
  let bytes = u.to_be_bytes().to_vec();
  encode_hex(&bytes)
}

pub fn u32_to_hex(u: &u32) -> String {
  let bytes = u.to_be_bytes().to_vec();
  encode_hex(&bytes)
}

use alloc::{string::String, vec::Vec};

use casper_contract::{
  contract_api::{runtime, storage},
  unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
  CLType, ContractPackageHash, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key,
  Parameter,
};

pub fn endpoint(
  name: &str,
  param: Vec<Parameter>,
  ret: CLType,
  access: Option<&str>,
) -> EntryPoint {
  EntryPoint::new(
    String::from(name),
    param,
    ret,
    match access {
      None => EntryPointAccess::Public,
      Some(access_key) => EntryPointAccess::groups(&[access_key]),
    },
    EntryPointType::Contract,
  )
}

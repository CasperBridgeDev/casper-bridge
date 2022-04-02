use casper_contract::contract_api::runtime;
use casper_types::{
  account::{Account, AccountHash},
  runtime_args, Contract, ContractHash, HashAddr, Key, RuntimeArgs, U256,
};

pub trait ERC20Trait {
  fn balance_of(&self, owner: AccountHash) -> U256;
  fn mint(&self, to: AccountHash, amount: U256);
  fn burn(&self, from: AccountHash, amount: U256);

  fn new(hash: ContractHash) -> Self;
}

pub struct SuperToken {
  pub hash: ContractHash,
}

impl ERC20Trait for SuperToken {
  fn new(hash: ContractHash) -> SuperToken {
    SuperToken { hash }
  }

  fn balance_of(&self, owner: AccountHash) -> U256 {
    runtime::call_contract(
      self.hash,
      "balance_of",
      runtime_args! {
        "owner" => Key::from(owner),
      },
    )
  }

  fn mint(&self, to: AccountHash, amount: U256) {
    let _: () = runtime::call_contract(
      self.hash,
      "mint",
      runtime_args! {
        "to" => Key::from(to),
        "amount" => amount,
      },
    );
  }

  fn burn(&self, from: AccountHash, amount: U256) {
    let _: () = runtime::call_contract(
      self.hash,
      "burn",
      runtime_args! {
        "from" => Key::from(from),
        "amount" => amount
      },
    );
  }
}

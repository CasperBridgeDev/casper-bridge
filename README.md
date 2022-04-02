![image](https://user-images.githubusercontent.com/102878074/161405147-c6d9710e-6a81-44ae-9517-e7bacd841ede.png)


# Casper Bridge (rust + solidity)

## What is presented:

- bridge-rs/contract (`make prepare && make build-contract`): bridge contract for casper in rust (in theory: suitable for any blockchain with public address less or eq than 40 bytes)
- bridge-rs/indexer `yarn && yarn generate && yarn migrate && yarn run-api` + `yarn run-indexer` : custom indexer for casper, indexes events from casper chain, serves as offchain database for validator to pick burn/mint events.
- bridge-rs/deployer (`yarn run`): useful during development.
- bridge-rs/tests (`cargo test`): bridge tests (not everything is covered)

- bridge-sol/bridge-subgraph `yarn && yarn codegen` : indexes events from evm chain (any is suitable: avalanche, fantom, polygon etc), serves as offchain database for validator to pick burn/mint events (depends on graphprotocol/graph-node).
- bridge-sol/sol-contract (compile with ethereum remix ide, or with solc): bridge contract in solidity (in theory: suitable for any blockchain with public address less or eq than 40 bytes)

if you need bridge only between evm chains, you can use `bridge-subgraph`.

## What is also can be presented, but not shared in the meantime (it depends on your support and activity, so you can run your own bridge on any chain you want ! ):

- front-end: serves as entry point for crypto application.
- validator: just listen for burn events (burn_and_create_proof) on source chain (from subgraph or indexer) and forward them to destination chain, where they eventualy approved with (approve_burn_proof) ; then after that user can mint without any issues with (mint_with_burn_proof) on destination chain.

P.S. be cautious, it needs to be audited, use it at your own risk !

MIT License

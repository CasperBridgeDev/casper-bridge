#!/bin/bash

# $1 = action (deploy, create, remove)
# $2 = chain

graph $1 --node http://localhost:8020/ --ipfs http://localhost:5001/ org/super-project-$2 ./subgraph.$2.yaml

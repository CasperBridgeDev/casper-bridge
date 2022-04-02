// SPDX-License-Identifier: MIT

pragma solidity ^0.8.13;

import "@openzeppelin/contracts/access/AccessControlEnumerable.sol";

interface IMintBurnBridgeToken {
    function balanceOf(address account) external returns (uint256);

    function mintByBridge(address to, uint256 amount) external;

    function burnByBridge(address from, uint256 amount) external;
}

error InvalidCallerLength();
error InvalidTokenLength();
error AllowanceNotFound();
error ProofIsNotApprovedOrAlreadyExecuted();
error PackageIsInvalid();
error ProvidedHashIsInvalid();
error ReduceAmount();



contract CrossChainBridge is AccessControlEnumerable {
    event ProofOfBurn(
        bytes mintToken,
        bytes burnToken,
        bytes mintCaller,
        bytes burnCaller,
        uint256 burnAmount,
        uint256 burnNonce,
        ChainType mintChainType,
        uint32 mintChainId,
        ChainType burnChainType,
        uint32 burnChainId,
        bytes32 burnProofHash
    );

    event ProofOfMint(
        bytes mintToken,
        bytes burnToken,
        bytes mintCaller,
        bytes burnCaller,
        uint256 burnAmount,
        ChainType mintChainType,
        uint32 mintChainId,
        ChainType burnChainType,
        uint32 burnChainId,
        bytes32 burnProofHash
    );

    event ApprovedBurnProof(bytes32 burnProofHash);

    enum States {
        DefaultValue,
        Burned,
        Approved,
        Executed
    }

    enum ChainType {
        Undefined,
        Evm,
        Casper,
        Solana,
        Radix
    }

    enum Allowance {
        Undefined,
        Allowed,
        Blocked
    }

    bytes32 public constant ROLE_APPROVER = keccak256("ROLE_APPROVER");

    ChainType thisChainType;

    // it's common storage for all chains (evm, casper, solana etc)
    mapping(bytes32 => States) public burnProofStorage;
    mapping(address => uint32) public nonceByToken;
    mapping(bytes32 => Allowance) public allowances;

    constructor() {
        _setupRole(ROLE_APPROVER, msg.sender);

        thisChainType = ChainType.Evm;
    }

    function setAllowance(
        bytes memory mintGenericToken,
        bytes memory burnGenericToken,
        ChainType mintChainType,
        uint32 mintChainId,
        ChainType burnChainType,
        uint32 burnChainId
    ) external onlyRole(ROLE_APPROVER) {
        bytes32 allowanceHash = getAllowanceHash(
            mintGenericToken,
            burnGenericToken,
            mintChainType,
            mintChainId,
            burnChainType,
            burnChainId
        );

        allowances[allowanceHash] = Allowance.Allowed;
    }

    function getAllowanceHash(
        bytes memory mintGenericToken,
        bytes memory burnGenericToken,
        ChainType mintChainType,
        uint32 mintChainId,
        ChainType burnChainType,
        uint32 burnChainId
    ) private view returns (bytes32) {
        bytes memory mintBytes = abi.encodePacked(
            mintGenericToken,
            mintChainType,
            mintChainId
        );
        bytes memory burnBytes = abi.encodePacked(
            burnGenericToken,
            burnChainType,
            burnChainId
        );

        if (sha256(mintBytes) > sha256(burnBytes)) {
            return sha256(abi.encodePacked(mintBytes, burnBytes));
        }

        return sha256(abi.encodePacked(burnBytes, mintBytes));
    }

    // 40 bytes
    function genericAddress(address some) private pure returns (bytes memory) {
        bytes memory result = new bytes(40);
        assembly {
            mstore(add(result, 40), some)
        }
        return result;
    }

    function approveBurnProof(bytes32 proofHash) external onlyRole(ROLE_APPROVER) {
        require(
            burnProofStorage[proofHash] == States.DefaultValue,
            "CCB: Already approved"
        );
        burnProofStorage[proofHash] = States.Approved;
        emit ApprovedBurnProof(proofHash);
    }

    function mintWithBurnProof(
        address mintToken,
        bytes memory burnGenericToken,
        bytes memory burnGenericCaller,
        uint burnChainType,
        uint32 burnChainId,
        uint256 burnAmount,
        uint256 burnNonce,
        bytes32 burnProofHash
    ) external {
        if (burnGenericToken.length != 40) {
            revert InvalidCallerLength();
        }
        if (burnGenericCaller.length != 40) {
            revert InvalidTokenLength();
        }

        bytes memory mintGenericCaller = genericAddress(msg.sender);
        bytes memory mintGenericToken = genericAddress(mintToken);

        {
            bytes32 allowanceHash = getAllowanceHash(
                mintGenericToken,
                burnGenericToken,
                thisChainType,
                uint32(block.chainid),
                ChainType(burnChainType),
                burnChainId
            );

            if (allowances[allowanceHash] != Allowance.Allowed) {
                revert AllowanceNotFound();
            }

        }

        if (burnProofStorage[burnProofHash] != States.Approved) {
            revert ProofIsNotApprovedOrAlreadyExecuted();
        }

        // prettier-ignore
        bytes memory packed = abi.encodePacked(
            mintGenericCaller, burnGenericCaller,
            mintGenericToken, burnGenericToken,
            burnAmount,
            uint8(thisChainType), uint32(block.chainid),
            burnChainType, burnChainId,
            burnNonce
        );

        if (burnProofStorage[burnProofHash] != States.Approved) {
            revert ProofIsNotApprovedOrAlreadyExecuted();
        }

        if (packed.length != 234) {
            revert PackageIsInvalid();
        }

        bytes32 computedBurnProofHash = sha256(packed);

        if (computedBurnProofHash != burnProofHash) {
            revert ProvidedHashIsInvalid();
        }

        burnProofStorage[burnProofHash] = States.Executed;

        IMintBurnBridgeToken(mintToken).mintByBridge(msg.sender, burnAmount);

        {
            ChainType _burnChainType = ChainType(burnChainType);
            uint32 _burnChainId = burnChainId;
            emit ProofOfMint(
                mintGenericToken,
                burnGenericToken,
                mintGenericCaller,
                burnGenericCaller,
                burnAmount,
                thisChainType,
                uint32(block.chainid),
                _burnChainType,
                _burnChainId,
                computedBurnProofHash
            );
        }
    }

    function burnAndCreateProof(
        address burnToken,
        bytes memory mintGenericToken,
        bytes memory mintGenericCaller,
        uint256 burnAmount,
        uint8 mintChainType,
        uint32 mintChainId
    ) external {
        if (mintGenericCaller.length != 40) {
            revert InvalidCallerLength();
        }
        if (mintGenericToken.length != 40) {
            revert InvalidTokenLength();
        }

        bytes memory burnGenericCaller = genericAddress(msg.sender);
        bytes memory burnGenericToken = genericAddress(burnToken);

        // stack to deep fix
        {
            bytes32 allowanceHash = getAllowanceHash(
                mintGenericToken,
                burnGenericToken,
                ChainType(mintChainType),
                mintChainId,
                ChainType(thisChainType),
                uint32(block.chainid)
            );

            if (allowances[allowanceHash] != Allowance.Allowed) {
                revert AllowanceNotFound();
            }
        }

        if (burnAmount > IMintBurnBridgeToken(burnToken).balanceOf(msg.sender)) {
            revert ReduceAmount();
        }

        uint256 burnNonce = nonceByToken[burnToken];

        // prettier-ignore
        bytes memory packed = abi.encodePacked(
            mintGenericCaller, burnGenericCaller,
            mintGenericToken, burnGenericToken,
            burnAmount,
            mintChainType, mintChainId,
            uint8(thisChainType), uint32(block.chainid),
            burnNonce
        );

        if (packed.length != 234) {
            revert PackageIsInvalid();
        }

        bytes32 computedBurnProofHash = sha256(packed);

        burnProofStorage[computedBurnProofHash] = States.Burned;

        IMintBurnBridgeToken(burnToken).burnByBridge(msg.sender, burnAmount);

        emit ProofOfBurn(
            mintGenericToken,
            burnGenericToken,
            mintGenericCaller,
            burnGenericCaller,
            burnAmount,
            burnNonce,
            ChainType(mintChainType),
            mintChainId,
            thisChainType,
            uint32(block.chainid),
            computedBurnProofHash
        );

        nonceByToken[burnToken]++;

        // return computedBurnProofHash;
    }
}

pragma solidity ^0.8.20;

import { ILayerZeroUltraLightNodeV2 } from "@layerzerolabs/lz-evm-v1-0.7/contracts/interfaces/ILayerZeroUltraLightNodeV2.sol";

import { WorkerMock as Worker } from "@layerzerolabs/test-devtools-evm-foundry/contracts/mocks/WorkerMock.sol";

import { ReadLib1002Mock as ReadLib1002 } from "@layerzerolabs/test-devtools-evm-foundry/contracts/mocks/ReadLib1002Mock.sol";
import { IDVN } from "@layerzerolabs/lz-evm-messagelib-v2/contracts/uln/interfaces/IDVN.sol";
import { IDVNFeeLib } from "@layerzerolabs/lz-evm-messagelib-v2/contracts/uln/interfaces/IDVNFeeLib.sol";
import { IReceiveUlnE2 } from "@layerzerolabs/lz-evm-messagelib-v2/contracts/uln/interfaces/IReceiveUlnE2.sol";

struct ExecuteParam {
    uint32 vid;
    address target;
    bytes callData;
    uint256 expiration;
    bytes signatures;
}

contract CustomDVN is Worker, IDVN {
    // to uniquely identify this DVN instance
    uint32 public immutable vid;
    uint32 public immutable localEidV2; // endpoint-v2 only, for read call

    mapping(uint32 dstEid => DstConfig) public dstConfig;
    mapping(bytes32 executableHash => bool used) public usedHashes;

    error DVN_OnlySelf();
    error DVN_InvalidRole(bytes32 role);
    error DVN_InstructionExpired();
    error DVN_InvalidTarget(address target);
    error DVN_InvalidVid(uint32 vid);
    error DVN_InvalidSignatures();
    error DVN_DuplicatedHash(bytes32 executableHash);

    event VerifySignaturesFailed(uint256 idx);
    event ExecuteFailed(uint256 _index, bytes _data);
    event HashAlreadyUsed(ExecuteParam param, bytes32 _hash);

    // packetHash is keccak256(packetHeader + payloadHash)
    event PacketAssigned(bytes32 packetHash, uint256 fee);

    // ========================= Constructor =========================

    /// @dev DVN doesn't have a roleAdmin (address(0x0))
    /// @dev Supports all of ULNv2, ULN301, ULN302 and more
    /// @param _localEidV2 local endpoint-v2 eid
    /// @param _vid unique identifier for this DVN instance
    /// @param _messageLibs array of message lib addresses that are granted the MESSAGE_LIB_ROLE
    /// @param _priceFeed price feed address
    /// @param _admins array of admin addresses that are granted the ADMIN_ROLE
    constructor(
        uint32 _localEidV2,
        uint32 _vid,
        address[] memory _messageLibs,
        address _priceFeed,
        address[] memory _admins
    ) Worker(_messageLibs, _priceFeed, 12000, address(0x0), _admins) {
        vid = _vid;
        localEidV2 = _localEidV2;
    }

    // ========================= Modifier =========================

    /// @dev depending on role, restrict access to only self or admin
    /// @dev ALLOWLIST, DENYLIST, MESSAGE_LIB_ROLE can only be granted/revoked by self
    /// @dev ADMIN_ROLE can only be granted/revoked by admin
    /// @dev reverts if not one of the above roles
    /// @param _role role to check
    modifier onlySelfOrAdmin(bytes32 _role) {
        if (_role == ALLOWLIST || _role == DENYLIST || _role == MESSAGE_LIB_ROLE) {
            // self required
            if (address(this) != msg.sender) {
                revert DVN_OnlySelf();
            }
        } else if (_role == ADMIN_ROLE) {
            // admin required
            _checkRole(ADMIN_ROLE);
        } else {
            revert DVN_InvalidRole(_role);
        }
        _;
    }

    modifier onlySelf() {
        if (address(this) != msg.sender) {
            revert DVN_OnlySelf();
        }
        _;
    }


    // ========================= OnlySelf / OnlyAdmin =========================

    /// @dev overrides AccessControl to allow self/admin to grant role'
    /// @dev function sig 0x2f2ff15d
    /// @param _role role to grant
    /// @param _account account to grant role to
    function grantRole(bytes32 _role, address _account) public override onlySelfOrAdmin(_role) {
        _grantRole(_role, _account);
    }

    /// @dev overrides AccessControl to allow self/admin to revoke role
    /// @dev function sig 0xd547741f
    /// @param _role role to revoke
    /// @param _account account to revoke role from
    function revokeRole(bytes32 _role, address _account) public override onlySelfOrAdmin(_role) {
        _revokeRole(_role, _account);
    }

    // ========================= OnlyAdmin =========================

    /// @param _params array of DstConfigParam
    function setDstConfig(DstConfigParam[] calldata _params) external onlyRole(ADMIN_ROLE) {
        for (uint256 i = 0; i < _params.length; ++i) {
            DstConfigParam calldata param = _params[i];
            dstConfig[param.dstEid] = DstConfig(param.gas, param.multiplierBps, param.floorMarginUSD);
        }
        emit SetDstConfig(_params);
    }

    /// @dev takes a list of instructions and executes them in order
    /// @dev if any of the instructions fail, it will emit an error event and continue to execute the rest of the instructions
    /// @param _params array of ExecuteParam, includes target, callData, expiration, signatures
    function execute(ExecuteParam[] calldata _params) external onlyRole(ADMIN_ROLE) {
        for (uint256 i = 0; i < _params.length; ++i) {
            ExecuteParam calldata param = _params[i];
            // 1. skip if invalid vid
            if (param.vid != vid) {
                continue;
            }

            // 2. skip if expired
            if (param.expiration <= block.timestamp) {
                continue;
            }

            // generate and validate hash
            bytes32 hash = hashCallData(param.vid, param.target, param.callData, param.expiration);

            // 3. check signatures
            // * add signature checking code *

            // 4. should check hash
            bool shouldCheckHash = _shouldCheckHash(bytes4(param.callData));
            if (shouldCheckHash) {
                if (usedHashes[hash]) {
                    emit HashAlreadyUsed(param, hash);
                    continue;
                } else {
                    usedHashes[hash] = true; // prevent reentry and replay attack
                }
            }

            (bool success, bytes memory rtnData) = param.target.call(param.callData);
            if (!success) {
                if (shouldCheckHash) {
                    // need to unset the usedHash otherwise it cant be used
                    usedHashes[hash] = false;
                }
                // emit an event in any case
                emit ExecuteFailed(i, rtnData);
            }
        }
    }

    // ========================= OnlyMessageLib =========================

    /// @dev for ULN301, ULN302 and more to assign job
    /// @dev dvn network can reject job from _sender by adding/removing them from allowlist/denylist
    /// @param _param assign job param
    /// @param _options dvn options
    function assignJob(
        AssignJobParam calldata _param,
        bytes calldata _options
    ) external payable onlyRole(MESSAGE_LIB_ROLE) onlyAcl(_param.sender) returns (uint256 totalFee) {
        totalFee = 0;
        emit PacketAssigned(
            keccak256(abi.encodePacked(_param.packetHeader, _param.payloadHash)),
            totalFee
        );
    }

    /// @dev to support ReadLib
    // @param _packetHeader - version + nonce + path
    // @param _cmd - the command to be executed to obtain the payload
    // @param _options - options
    function assignJob(
        address _sender,
        bytes calldata /*_packetHeader*/,
        bytes calldata _cmd,
        bytes calldata _options
    ) external payable onlyRole(MESSAGE_LIB_ROLE) onlyAcl(_sender) returns (uint256 fee) {
        fee = 0;
    }

    // ========================= View =========================

    /// @dev getFee can revert if _sender doesn't pass ACL
    /// @param _dstEid destination EndpointId
    /// @param _confirmations block confirmations
    /// @param _sender message sender address
    /// @param _options dvn options
    /// @return fee fee in native amount
    function getFee(
        uint32 _dstEid,
        uint64 _confirmations,
        address _sender,
        bytes calldata _options
    ) external view returns (uint256 fee) {
        return 0;
    }

    /// @dev to support ReadLib
    // @param _packetHeader - version + nonce + path
    // @param _cmd - the command to be executed to obtain the payload
    // @param _options - options
    function getFee(
        address _sender,
        bytes calldata /*_packetHeader*/,
        bytes calldata _cmd,
        bytes calldata _options
    ) external view returns (uint256 fee) {
        return 0;
    }

    /// @param _target target address
    /// @param _callData call data
    /// @param _expiration expiration timestamp
    /// @return hash of above
    function hashCallData(
        uint32 _vid,
        address _target,
        bytes calldata _callData,
        uint256 _expiration
    ) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(_vid, _target, _expiration, _callData));
    }

    // ========================= Internal =========================

    /// @dev to save gas, we don't check hash for some functions (where replaying won't change the state)
    /// @dev for example, some administrative functions like changing signers, the contract should check hash to double spending
    /// @dev should ensure that all onlySelf functions have unique functionSig
    /// @param _functionSig function signature
    /// @return true if should check hash
    function _shouldCheckHash(bytes4 _functionSig) internal pure returns (bool) {
        // never check for these selectors to save gas
        return
            _functionSig != IReceiveUlnE2.verify.selector && // 0x0223536e, replaying won't change the state
            _functionSig != ReadLib1002.verify.selector && // 0xab750e75, replaying won't change the state
            _functionSig != ILayerZeroUltraLightNodeV2.updateHash.selector; // 0x704316e5, replaying will be revert at uln
    }
}

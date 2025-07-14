// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// A very simple mock Guardrail contract, this would be replaced with Safe (or similar) contract
/// in production. In this mock, it only allows the owner to forward calls to other contracts.
contract MockGuardrail is Ownable {
    constructor(address initialOwner) Ownable(initialOwner) {}

    function forwardCall(address target, bytes calldata data) external onlyOwner returns (bytes memory returnData) {
        (bool success, bytes memory _returnData) = target.call(data);
        require(success, "Forward call failed");
        return _returnData;
    }
}

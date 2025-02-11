// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

interface IBVSDriver {
    function executeBVSOffchain(string memory taskId) external;
    function addRegisteredBVSContract(address contractAddress) external;
}
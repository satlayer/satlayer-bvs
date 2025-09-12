// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IPL {
    function repayAndRestake(address user, uint256 assets, bytes32 strategy) external;
    function finalizeUnwind(address user, bytes32 strategy) external;
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IPriceOracle {
    function price(address token) external view returns (uint256); // 1e8
}

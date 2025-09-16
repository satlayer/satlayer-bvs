// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IWrapper1to1 {
    function base() external view returns (address);
    function wrapped() external view returns (address);
    function wrap(uint256 amount) external returns (uint256 out); // expect out == amount (1:1)
    function unwrap(uint256 amount) external returns (uint256 out); // expect out == amount (1:1)
}

// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

interface IStateBank {
    function set(string memory key, string memory value) external returns (bool);
    function get(string memory key) external view returns (string memory);
    function addRegisteredBVSContract(address contractAddress) external returns (bool);
}
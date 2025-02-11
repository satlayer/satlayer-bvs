// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

interface IBVSSquaring {
    function createNewTask(string memory input) external;

    function respondToTask(uint256 taskId, int256 result, string memory operators) external;

    function getTaskInput(uint256 taskId) external view returns (string memory);

    function getTaskResult(uint256 taskId) external view returns (int256);
    
    function getLatestTaskId() external view returns (uint256);
}
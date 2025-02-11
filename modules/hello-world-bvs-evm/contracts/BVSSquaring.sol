// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {Ownable2StepUpgradeable} from "@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";

import {IBVSSquaring} from "./interfaces/IBVSSquaring.sol";
import {IBVSDriver} from "./interfaces/IBVSDriver.sol";
import {IStateBank} from "./interfaces/IStateBank.sol";

contract BVSSquaring is Initializable, Ownable2StepUpgradeable, IBVSSquaring {
    using Strings for uint256;
    using Strings for int256;

    // ============================== Variables ============================== //

    IBVSDriver public bvsDriver;
    IStateBank  public stateBank;

    address public aggregator;

    uint256 public maxId;

    /// Mapping from taskId -> input value
    mapping(uint256 => string) public tasks;

    /// Mapping from taskId -> result (once aggregator responds).
    /// If a taskId does not exist here, it has not yet been responded to.
    mapping(uint256 => int256) public respondedTasks;


    // ============================== Events ============================== //

    event CreateNewTask(uint256 indexed taskId, string input);

    event TaskResponded(
        uint256 indexed taskId,
        int256 result,
        string operators
    );

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    // ============================== Initialization ============================== //

    /**
     * @dev Initialize with references to bvsDriver, stateBank, and aggregator.
     *      This mirrors the Rust contract storing aggregator, state_bank, bvs_driver.
     *
     * @param _bvsDriver   The address of the deployed BvsDriver.
     * @param _stateBank   The address of the deployed StateBank.
     * @param _aggregator  The aggregator address who can respond to tasks.
     */
    function initialize(
        address _bvsDriver,
        address _stateBank,
        address _aggregator
    ) external initializer {
        require(_bvsDriver != address(0),  "bvsDriver cannot be zero");
        require(_stateBank != address(0),  "stateBank cannot be zero");
        require(_aggregator != address(0), "aggregator cannot be zero");

        __Ownable2Step_init();
        _transferOwnership(_msgSender());

        bvsDriver = IBVSDriver(_bvsDriver);
        stateBank = IStateBank(_stateBank);
        aggregator = _aggregator;
    }

    // ============================== External Functions ============================== //

    /**
     * @notice Creates a new task with `input`.
     *         - increments `maxId`
     *         - stores the task input
     *         - calls `stateBank.set("taskId.<maxId>", <input>)`
     *         - calls `bvsDriver.executeBvsOffchain(<maxId>)`
     *         - emits `CreateNewTask` event
     *
     * This mirrors the Rust contract’s `create_new_task`.
     */
    function createNewTask(string memory input) external {
        maxId += 1;
        uint256 newId = maxId;
    
        tasks[newId] = input;
    
        string memory newIdStr = newId.toString(); 
        string memory key = string(abi.encodePacked("taskId.", newIdStr));
        string memory value = input;
    
        stateBank.set(key, value);
    
        bvsDriver.executeBVSOffchain(newIdStr);
    
        emit CreateNewTask(newId, input);
    }
    

    /**
     * @notice Respond to a task with the aggregator’s result.
     *         - only the aggregator can call this
     *         - ensures the task hasn’t already been responded to
     *         - stores the result
     *         - emits `TaskResponded` event
     *
     * Mirrors the Rust contract’s `respond_to_task`.
     *
     * @param taskId    The ID of the task to respond to
     * @param result    The aggregator’s computed result
     * @param operators A address representing which operators contributed, for example
     */
    function respondToTask(
        uint256 taskId,
        int256 result,
        string memory operators
    ) external {
        require(msg.sender == aggregator, "Unauthorized aggregator");
        require(_notResponded(taskId), "ResultAlreadySubmitted");

        respondedTasks[taskId] = result;

        emit TaskResponded(taskId, result, operators);
    }

    // ============================== Internal Functions ============================== //

    function _notResponded(uint256 taskId) internal view returns (bool) {
        return respondedTasks[taskId] == int256(0);
    }

    // ============================== Getters Functions ============================== //

    function getTaskInput(uint256 taskId) external view returns (string memory) {
        require(bytes(tasks[taskId]).length != 0, "NoValueFound");
        return tasks[taskId];
    }

    /**
     * @notice Returns the aggregator’s responded result for a given taskId, or reverts if not responded.
     */
    function getTaskResult(uint256 taskId) external view returns (int256) {
        int256 stored = respondedTasks[taskId];
        require(stored != int256(0), "NoValueFound");
        return stored;
    }

    function getLatestTaskId() external view returns (uint256) {
        return maxId;
    }
}

// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Test, console} from "forge-std/Test.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYRegistryTest is Test, TestSuite {
    function test() public view {
        assertEq(registry.owner(), owner);
        assertEq(registry.paused(), false);
    }
}

// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {SLAYTestSuite} from "./SLAYTestSuite.sol";

contract SLAYRegistryTest is Test, SLAYTestSuite {
    function test() public view {
        assertEq(registry.owner(), owner);
        assertEq(registry.paused(), false);
    }
}

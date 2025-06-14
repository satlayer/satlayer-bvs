// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYRouterTest is Test, TestSuite {
    function test() public view {
        assertEq(router.owner(), owner);
        assertEq(router.paused(), false);
    }
}

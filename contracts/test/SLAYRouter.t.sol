// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Test, console} from "forge-std/Test.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYRouterTest is Test, TestSuite {
    function test() public view {
        assertEq(router.owner(), owner);
        assertEq(router.paused(), false);
    }
}

// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

//import ".pnpm/forge-std@https+++github.com+foundry-rs+forge-std+tarball+v1.9.7/node_modules/forge-std/src/Script.sol";
import {Script, console} from "forge-std/Script.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

contract DeploymentScript is Script {
    function run() public {
        console.log("Script");
    }
}

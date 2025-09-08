// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {Script, console} from "forge-std/Script.sol";
import {ISLAYRegistryV2} from "../src/interface/ISLAYRegistryV2.sol";
import {ISLAYRouterV2} from "../src/interface/ISLAYRouterV2.sol";
import {ISLAYVaultFactoryV2} from "../src/interface/ISLAYVaultFactoryV2.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";

contract SepoliaContracts is Script {
    ISLAYRouterV2 public router = ISLAYRouterV2(0xfcFd849e8F28EA01b4B7A9Bb170C63C4DF3A8466);
    ISLAYRegistryV2 public registry = ISLAYRegistryV2(0xc44031bc45F39E3B7221F75cfa6BC3589636C622);
    ISLAYVaultFactoryV2 public factory = ISLAYVaultFactoryV2(0xcC0Ede3FB35F783095f8823f27111c5a7E573973);

    function promptConfirm() internal {
        string memory confirm = vm.prompt("Confirm (Y/n)");
        require(keccak256(bytes(confirm)) == keccak256("Y"), "Operation cancelled.");
    }
}

contract RegisterOperator is SepoliaContracts {
    function run() public virtual {
        require(block.chainid == 11155111, "You're not on Sepolia");

        string memory name = vm.prompt("Name of the Operator");
        string memory uri = vm.prompt("URL of the Operator");

        vm.startBroadcast();
        console.log("Name:   ", name);
        console.log("Uri:    ", uri);
        console.log("Sender: ", msg.sender);
        promptConfirm();

        registry.registerAsOperator(uri, name);
    }
}

contract RegisterService is SepoliaContracts {
    function run() public virtual {
        require(block.chainid == 11155111, "You're not on Sepolia");

        string memory name = vm.prompt("Name of the Service");
        string memory uri = vm.prompt("URL of the Service");

        vm.startBroadcast();
        console.log("Name:   ", name);
        console.log("Uri:    ", uri);
        console.log("Sender: ", msg.sender);
        promptConfirm();

        registry.registerAsService(uri, name);
    }
}

contract LinkService is SepoliaContracts {
    function run() public virtual {
        require(block.chainid == 11155111, "You're not on Sepolia");

        address service = vm.promptAddress("Address of the Service");

        vm.startBroadcast();
        console.log("Service              :", service);
        console.log("Operator (msg.sender):", msg.sender);
        promptConfirm();

        registry.registerServiceToOperator(service);
    }
}

contract LinkOperator is SepoliaContracts {
    function run() public virtual {
        require(block.chainid == 11155111, "You're not on Sepolia");

        address operator = vm.promptAddress("Address of the Operator");

        vm.startBroadcast();
        console.log("Operator             :", operator);
        console.log("Service  (msg.sender):", msg.sender);
        promptConfirm();

        registry.registerOperatorToService(operator);
    }
}

contract DeployVault is SepoliaContracts {
    function run() public virtual {
        require(block.chainid == 11155111, "You're not on Sepolia");

        address asset = vm.promptAddress("ERC20 Asset");
        string memory name = vm.prompt("Infix name of the receipt token");
        string memory symbol = vm.prompt("Infix symbol of the receipt token");

        vm.startBroadcast();
        console.log("Operator  (msg.sender):", msg.sender);
        promptConfirm();

        IERC20Metadata metadata = IERC20Metadata(asset);
        // Using self-serve vault creation
        factory.create(metadata, name, symbol);
    }
}

// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {SLAYDeployment} from "./Deployment.s.sol";
import {ISLAYRegistryV2} from "../src/interface/ISLAYRegistryV2.sol";
import {SLAYRegistryV2} from "../src/SLAYRegistryV2.sol";
import {SLAYRewardsV2} from "../src/SLAYRewardsV2.sol";

import {SLAYRouterV2} from "../src/SLAYRouterV2.sol";
import {SLAYVaultFactoryV2} from "../src/SLAYVaultFactoryV2.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";

contract SLAYNet is SLAYDeployment {
    SmartAccount[] public services;
    SmartAccount[] public operators;

    /// @dev Deploy SLAYNet script.
    /// forge script SLAYDeployment --rpc-url slaynet --slow --broadcast --verify
    function run() public virtual override {
        vm.startBroadcast();
        (SLAYRouterV2 router, SLAYRegistryV2 registry, SLAYVaultFactoryV2 vaultFactory, SLAYRewardsV2 rewards) =
            deploy(msg.sender);

        deployServices(registry);
        deployOperators(registry);
        establishRelationships(registry);
        deployVaults(vaultFactory, router);
    }

    function deployServices(SLAYRegistryV2 registry) public virtual {
        {
            SmartAccount service = new SmartAccount(msg.sender);
            services.push(service);

            service.exec(
                address(registry),
                abi.encodeCall(
                    SLAYRegistryV2.registerAsService, ("https://forwardminer.slaynet.satlayer.xyz", "Forward Miner")
                )
            );
        }

        {
            SmartAccount service = new SmartAccount(msg.sender);
            services.push(service);

            service.exec(
                address(registry),
                abi.encodeCall(
                    SLAYRegistryV2.registerAsService, ("https://insuranceplus.slaynet.satlayer.xyz", "Insurance Plus")
                )
            );

            SmartAccount destination = new SmartAccount(msg.sender);
            ISLAYRegistryV2.SlashParameter memory parameter = ISLAYRegistryV2.SlashParameter({
                destination: address(destination),
                maxMbips: 100_000,
                resolutionWindow: 3600
            });
            service.exec(address(registry), abi.encodeCall(SLAYRegistryV2.enableSlashing, (parameter)));

            service.exec(address(registry), abi.encodeCall(SLAYRegistryV2.setMinWithdrawalDelay, (14 days)));
        }

        {
            SmartAccount service = new SmartAccount(msg.sender);
            services.push(service);

            service.exec(
                address(registry),
                abi.encodeCall(
                    SLAYRegistryV2.registerAsService,
                    ("https://slashprotection.slaynet.satlayer.xyz", "Slash Protection")
                )
            );

            SmartAccount destination = new SmartAccount(msg.sender);
            ISLAYRegistryV2.SlashParameter memory parameter = ISLAYRegistryV2.SlashParameter({
                destination: address(destination),
                maxMbips: 1_000_000,
                resolutionWindow: 7200
            });
            service.exec(address(registry), abi.encodeCall(SLAYRegistryV2.enableSlashing, (parameter)));
        }
    }

    function deployOperators(SLAYRegistryV2 registry) public virtual {
        {
            SmartAccount operator = new SmartAccount(msg.sender);
            operators.push(operator);

            operator.exec(
                address(registry),
                abi.encodeCall(
                    SLAYRegistryV2.registerAsOperator, ("https://ongrid.slaynet.satlayer.xyz", "OnGrid Operator")
                )
            );
        }

        {
            SmartAccount operator = new SmartAccount(msg.sender);
            operators.push(operator);

            operator.exec(
                address(registry),
                abi.encodeCall(SLAYRegistryV2.registerAsOperator, ("https://rainbow.slaynet.satlayer.xyz", "Rainbow"))
            );
            operator.exec(address(registry), abi.encodeCall(SLAYRegistryV2.setWithdrawalDelay, (uint32(10 days))));
        }
    }

    function establishRelationships(SLAYRegistryV2 registry) public virtual {
        // Operator[0] <-> Service[0]
        operators[0].exec(
            address(registry), abi.encodeCall(SLAYRegistryV2.registerServiceToOperator, (address(services[0])))
        );
        services[0].exec(
            address(registry), abi.encodeCall(SLAYRegistryV2.registerOperatorToService, (address(operators[0])))
        );
        // Operator[0] <-> Service[1]
        operators[0].exec(
            address(registry), abi.encodeCall(SLAYRegistryV2.registerServiceToOperator, (address(services[1])))
        );
        services[1].exec(
            address(registry), abi.encodeCall(SLAYRegistryV2.registerOperatorToService, (address(operators[0])))
        );
        // Operator[0] <-> Service[2]
        operators[0].exec(
            address(registry), abi.encodeCall(SLAYRegistryV2.registerServiceToOperator, (address(services[2])))
        );
        services[2].exec(
            address(registry), abi.encodeCall(SLAYRegistryV2.registerOperatorToService, (address(operators[0])))
        );
        // Operator[1] <-> Service[1]
        operators[1].exec(
            address(registry), abi.encodeCall(SLAYRegistryV2.registerServiceToOperator, (address(services[1])))
        );
        services[1].exec(
            address(registry), abi.encodeCall(SLAYRegistryV2.registerOperatorToService, (address(operators[1])))
        );
        // Operator[1] <-> Service[2]
        operators[1].exec(
            address(registry), abi.encodeCall(SLAYRegistryV2.registerServiceToOperator, (address(services[2])))
        );
        services[2].exec(
            address(registry), abi.encodeCall(SLAYRegistryV2.registerOperatorToService, (address(operators[1])))
        );
    }

    function deployVaults(SLAYVaultFactoryV2 vaultFactory, SLAYRouterV2 router) public virtual {
        IERC20Metadata wbtc = IERC20Metadata(0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599);
        IERC20Metadata lido = IERC20Metadata(0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84);
        IERC20Metadata cbbtc = IERC20Metadata(0xcbB7C0000aB88B473b1f5aFd9ef808440eed33Bf);

        bytes memory data;

        data = operators[0].exec(address(vaultFactory), abi.encodeWithSignature("create(address)", (wbtc)));
        router.setVaultWhitelist(abi.decode(data, (address)), true);

        data = operators[0].exec(address(vaultFactory), abi.encodeWithSignature("create(address)", (lido)));
        router.setVaultWhitelist(abi.decode(data, (address)), true);

        data = operators[0].exec(address(vaultFactory), abi.encodeWithSignature("create(address)", (cbbtc)));
        router.setVaultWhitelist(abi.decode(data, (address)), true);

        data = operators[1].exec(address(vaultFactory), abi.encodeWithSignature("create(address)", (wbtc)));
        router.setVaultWhitelist(abi.decode(data, (address)), true);

        data = operators[1].exec(address(vaultFactory), abi.encodeWithSignature("create(address)", (cbbtc)));
        router.setVaultWhitelist(abi.decode(data, (address)), true);
    }
}

/**
 * @dev Smart Account for SLAYNet,
 * to allow a singular parent owner to interact with multiple contracts without being the same caller.
 */
contract SmartAccount is Ownable {
    constructor(address initialOwner) Ownable(initialOwner) {}

    function exec(address target, bytes calldata data) external onlyOwner returns (bytes memory returnData) {
        (bool success, bytes memory _returnData) = target.call(data);
        require(success, "Forward call failed");
        return _returnData;
    }
}

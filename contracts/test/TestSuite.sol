// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {MockERC20} from "./MockERC20.sol";
import {EmptyImpl} from "../src/EmptyImpl.sol";
import {SLAYRegistry} from "../src/SLAYRegistry.sol";
import {SLAYRouter} from "../src/SLAYRouter.sol";
import {SLAYVaultFactory} from "../src/SLAYVaultFactory.sol";
import {SLAYVault} from "../src/SLAYVault.sol";
import {Test, console} from "forge-std/Test.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";

/**
 * @dev set up all the contracts needed for testing.
 */
contract TestSuite is Test {
    address public owner = vm.randomAddress();

    EmptyImpl public emptyImpl = new EmptyImpl();

    SLAYRouter public router;
    SLAYRegistry public registry;
    SLAYVaultFactory public vaultFactory;

    function setUp() public virtual {
        bytes memory emptyData = abi.encodeCall(EmptyImpl.initialize, (owner));

        router = SLAYRouter(UnsafeUpgrades.deployUUPSProxy(address(emptyImpl), emptyData));
        registry = SLAYRegistry(UnsafeUpgrades.deployUUPSProxy(address(emptyImpl), emptyData));

        SLAYVault vaultImpl = new SLAYVault(router, registry);
        address beacon = UnsafeUpgrades.deployBeacon(address(vaultImpl), owner);
        SLAYVaultFactory factoryImpl = new SLAYVaultFactory(beacon);
        vaultFactory = SLAYVaultFactory(
            UnsafeUpgrades.deployUUPSProxy(address(factoryImpl), abi.encodeCall(SLAYVaultFactory.initialize, (owner)))
        );

        vm.startPrank(owner);
        UnsafeUpgrades.upgradeProxy(
            address(router), address(new SLAYRouter(registry)), abi.encodeCall(SLAYRouter.initialize, ())
        );
        UnsafeUpgrades.upgradeProxy(
            address(registry), address(new SLAYRegistry(router)), abi.encodeCall(SLAYRegistry.initialize, ())
        );
        vm.stopPrank();
    }

    /**
     * @dev Create a new SLAYVault instance for testing with default parameters.
     * The vault will use a MockERC20 token with name "Token", symbol "TKN", and 18 decimals.
     * The caller must be an operator.
     * Use vm.startPrank() to set the operator before calling this function.
     */
    function newVault() public virtual returns (SLAYVault) {
        return newVault("Token", "TKN", 18);
    }

    /**
     * @dev Create a new SLAYVault instance for testing.
     * Params _name, _symbol, and _decimals are used to create a MockERC20 token
     * which serves as the underlying asset for the vault.
     * The caller must be an operator.
     * Use vm.startPrank() to set the operator before calling this function.
     */
    function newVault(string memory _name, string memory _symbol, uint8 _decimals) public virtual returns (SLAYVault) {
        MockERC20 underlying = new MockERC20(_name, _symbol, _decimals);
        address proxy = vaultFactory.create(underlying);
        return SLAYVault(proxy);
    }
}

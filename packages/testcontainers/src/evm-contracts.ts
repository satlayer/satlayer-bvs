import UpgradeableBeacon from "@openzeppelin/upgrades-core/artifacts/@openzeppelin/contracts-v5/proxy/beacon/UpgradeableBeacon.sol/UpgradeableBeacon.json";
import ERC1967Proxy from "@openzeppelin/upgrades-core/artifacts/@openzeppelin/contracts-v5/proxy/ERC1967/ERC1967Proxy.sol/ERC1967Proxy.json";
import mockPyth from "@satlayer/contracts/out/MockPyth.sol/MockPyth.json";
import slayBase from "@satlayer/contracts/out/SLAYBase.sol/SLAYBase.json";
import slayOracle from "@satlayer/contracts/out/SLAYOracle.sol/SLAYOracle.json";
import slayRegistry from "@satlayer/contracts/out/SLAYRegistryV2.sol/SLAYRegistryV2.json";
import slayRewards from "@satlayer/contracts/out/SLAYRewardsV2.sol/SLAYRewardsV2.json";
import slayRouter from "@satlayer/contracts/out/SLAYRouterV2.sol/SLAYRouterV2.json";
import slayVaultFactory from "@satlayer/contracts/out/SLAYVaultFactoryV2.sol/SLAYVaultFactoryV2.json";
import slayVault from "@satlayer/contracts/out/SLAYVaultV2.sol/SLAYVaultV2.json";
import { Account, encodeDeployData, encodeFunctionData, getContract, GetContractReturnType } from "viem";

import { saltToHex, StartedAnvilContainer, SuperTestClient } from "./anvil-container";
import erc20Abi from "./MockERC20.sol/MockERC20.json";

export class EVMContracts {
  public oracle: GetContractReturnType<typeof slayOracle.abi, SuperTestClient> | undefined;

  public mockPyth: GetContractReturnType<typeof mockPyth.abi, SuperTestClient> | undefined;

  private constructor(
    public readonly started: StartedAnvilContainer,
    public readonly registry: GetContractReturnType<typeof slayRegistry.abi, SuperTestClient>,
    public readonly router: GetContractReturnType<typeof slayRouter.abi, SuperTestClient>,
    public readonly rewards: GetContractReturnType<typeof slayRewards.abi, SuperTestClient>,
    public readonly vaultFactory: GetContractReturnType<typeof slayVaultFactory.abi, SuperTestClient>,
  ) {}

  get client(): SuperTestClient {
    return this.started.getClient();
  }

  get wallet(): Account {
    return this.started.getAccount();
  }

  static async bootstrap(started: StartedAnvilContainer): Promise<EVMContracts> {
    const owner = started.getAccount().address;

    // deploy SLAYBase impl contract
    const base = await started.deployContract({
      abi: slayBase.abi,
      bytecode: slayBase.bytecode.object as `0x${string}`,
      salt: "SLAYBase",
      constructorArgs: [],
    });

    // ================  Deploy Proxy Contracts ================

    // deploy registryProxy contract with ERC1967Proxy and SLAYBase as the impl contract
    const registryProxy = await started.deployContract({
      abi: ERC1967Proxy.abi,
      bytecode: ERC1967Proxy.bytecode as `0x${string}`,
      salt: "registryProxy",
      constructorArgs: [
        base.contractAddress,
        encodeFunctionData({
          abi: slayBase.abi,
          functionName: "initialize",
          args: [owner],
        }),
      ],
    });

    // deploy routerProxy contract with ERC1967Proxy and SLAYBase as the impl contract
    const routerProxy = await started.deployContract({
      abi: ERC1967Proxy.abi,
      bytecode: ERC1967Proxy.bytecode as `0x${string}`,
      salt: "routerProxy",
      constructorArgs: [
        base.contractAddress,
        encodeFunctionData({
          abi: slayBase.abi,
          functionName: "initialize",
          args: [owner],
        }),
      ],
    });

    // deploy rewardsProxy contract with ERC1967Proxy and SLAYBase as the impl contract
    const rewardsProxy = await started.deployContract({
      abi: ERC1967Proxy.abi,
      bytecode: ERC1967Proxy.bytecode as `0x${string}`,
      salt: "rewardsProxy",
      constructorArgs: [
        base.contractAddress,
        encodeFunctionData({
          abi: slayBase.abi,
          functionName: "initialize",
          args: [owner],
        }),
      ],
    });

    // deploy vaultFactory contract with SLAYBase as the impl contract
    const vaultFactory = await started.deployContract({
      abi: ERC1967Proxy.abi,
      bytecode: ERC1967Proxy.bytecode as `0x${string}`,
      salt: "vaultFactory",
      constructorArgs: [
        base.contractAddress,
        encodeFunctionData({
          abi: slayBase.abi,
          functionName: "initialize",
          args: [owner],
        }),
      ],
    });

    // ================  Deploy Implementation Contracts ================

    // deploy SLAYRegistryV2 impl contract
    const registryImpl = await started.deployContract({
      abi: slayRegistry.abi,
      bytecode: slayRegistry.bytecode.object as `0x${string}`,
      salt: "SLAYRegistryV2",
      constructorArgs: [routerProxy.contractAddress],
    });

    // deploy SLAYRouterV2 impl contract
    const routerImpl = await started.deployContract({
      abi: slayRouter.abi,
      bytecode: slayRouter.bytecode.object as `0x${string}`,
      salt: "SLAYRouterV2",
      constructorArgs: [registryProxy.contractAddress],
    });

    // deploy SLAYRewardsV2 impl contract
    const rewardsImpl = await started.deployContract({
      abi: slayRewards.abi,
      bytecode: slayRewards.bytecode.object as `0x${string}`,
      salt: "SLAYRewardsV2",
      constructorArgs: [],
    });

    // deploy SLAYVaultV2 impl contract
    const vaultImpl = await started.deployContract({
      abi: slayVault.abi,
      bytecode: slayVault.bytecode.object as `0x${string}`,
      salt: "SLAYVaultV2",
      constructorArgs: [routerProxy.contractAddress, registryProxy.contractAddress],
    });

    // deploy SLAYVaultV2 Beacon contract
    const vaultBeacon = await started.deployContract({
      abi: UpgradeableBeacon.abi,
      bytecode: UpgradeableBeacon.bytecode as `0x${string}`,
      salt: "SLAYVaultBeacon",
      constructorArgs: [vaultImpl.contractAddress, owner],
    });

    // deploy SLAYVaultFactoryV2 impl contract
    const vaultFactoryImpl = await started.deployContract({
      abi: slayVaultFactory.abi,
      bytecode: slayVaultFactory.bytecode.object as `0x${string}`,
      salt: "SLAYVaultFactoryV2",
      constructorArgs: [vaultBeacon.contractAddress, registryProxy.contractAddress],
    });

    // ================  Upgrade Proxy Contracts ================

    // get registryProxy contract instance ( cannot use registryProxy.contract because it is the instance of ERC1967Proxy )
    const registryProxyContract = getContract({
      address: registryProxy.contractAddress,
      abi: slayBase.abi,
      client: started.getClient(),
    });
    // upgrade registryProxy to use SLAYRegistryV2 impl contract
    await registryProxyContract.write.upgradeToAndCall([
      registryImpl.contractAddress,
      encodeFunctionData({
        abi: slayRegistry.abi,
        functionName: "initialize2",
        args: [],
      }),
    ]);
    // get registry contract instance
    const registryContract = getContract({
      address: registryProxy.contractAddress,
      abi: slayRegistry.abi,
      client: started.getClient(),
    });

    // get routerProxy contract instance ( cannot use routerProxy.contract because it is the instance of ERC1967Proxy )
    const routerProxyContract = getContract({
      address: routerProxy.contractAddress,
      abi: slayBase.abi,
      client: started.getClient(),
    });
    // upgrade routerProxy to use SLAYRouterV2 impl contract
    await routerProxyContract.write.upgradeToAndCall([
      routerImpl.contractAddress,
      encodeFunctionData({
        abi: slayRouter.abi,
        functionName: "initialize2",
        args: [],
      }),
    ]);
    // get router contract instance
    const routerContract = getContract({
      address: routerProxy.contractAddress,
      abi: slayRouter.abi,
      client: started.getClient(),
    });

    // get rewardsProxy contract instance ( cannot use rewardsProxy.contract because it is the instance of ERC1967Proxy )
    const rewardsProxyContract = getContract({
      address: rewardsProxy.contractAddress,
      abi: slayBase.abi,
      client: started.getClient(),
    });
    // upgrade rewardsProxy to use SLAYRewardsV2 impl contract
    await rewardsProxyContract.write.upgradeToAndCall([rewardsImpl.contractAddress, ""]);
    // get rewards contract instance
    const rewardsContract = getContract({
      address: rewardsProxy.contractAddress,
      abi: slayRewards.abi,
      client: started.getClient(),
    });

    // get vaultFactory contract instance ( cannot use vaultFactory.contract because it is the instance of ERC1967Proxy )
    const vaultFactoryProxyContract = getContract({
      address: vaultFactory.contractAddress,
      abi: slayBase.abi,
      client: started.getClient(),
    });
    // upgrade vaultFactory to use SLAYVaultFactoryV2 impl contract
    await vaultFactoryProxyContract.write.upgradeToAndCall([vaultFactoryImpl.contractAddress, ""]);
    // get vaultFactory contract instance
    const vaultFactoryContract = getContract({
      address: vaultFactory.contractAddress,
      abi: slayVaultFactory.abi,
      client: started.getClient(),
    });

    // mine a block to ensure all transactions are processed
    await started.mineBlock(1);

    return new EVMContracts(started, registryContract, routerContract, rewardsContract, vaultFactoryContract);
  }

  /// Initializes an ERC20 token with the given name, symbol, and decimals.
  async initERC20({ name, symbol, decimals }: { name: string; symbol: string; decimals: number }) {
    return this.started.deployContract({
      abi: erc20Abi.abi,
      bytecode: erc20Abi.bytecode.object as `0x${string}`,
      salt: `ERC20-${symbol}`,
      constructorArgs: [name, symbol, decimals],
    });
  }

  /// Initializes a SLAYVault for the given operator and underlying asset.
  /// Returns the address of the created SLAYVault.
  async initVault({
    operator,
    underlyingAsset,
  }: {
    operator: Account;
    underlyingAsset: `0x${string}`;
  }): Promise<`0x${string}`> {
    // get vault address from vault factory
    const createRes = await this.vaultFactory.simulate.create([underlyingAsset], { account: operator.address });
    // commit the transaction to create the vault
    await this.vaultFactory.write.create([underlyingAsset], { account: operator });
    return createRes.result;
  }

  /// Returns a contract instance for the SLAYVault at the given address.
  async getVaultContractInstance(
    vaultAddress: `0x${string}`,
  ): Promise<GetContractReturnType<typeof slayVault.abi, SuperTestClient>> {
    return getContract({
      address: vaultAddress,
      abi: slayVault.abi,
      client: this.started.getClient(),
    });
  }

  /// Initializes a SLAYOracle
  async initOracle() {
    const owner = this.started.getAccount().address;

    // deploy mock Pyth contract
    const mockPythContract = await this.started.deployContract({
      abi: mockPyth.abi,
      bytecode: mockPyth.bytecode.object as `0x${string}`,
      salt: "MockPyth",
      constructorArgs: [60, 1],
    });

    this.mockPyth = getContract({
      address: mockPythContract.contractAddress,
      abi: mockPyth.abi,
      client: this.started.getClient(),
    });

    // deploy SLAYBase impl contract
    const baseContractAddress = StartedAnvilContainer.getCreate2Address({
      deployBytecode: encodeDeployData({
        abi: slayBase.abi,
        args: [],
        bytecode: slayBase.bytecode.object as `0x${string}`,
      }),
      saltHex: saltToHex("SLAYBase"),
    });

    // deploy oracleProxy contract with ERC1967Proxy and SLAYBase as the impl contract
    const oracleProxy = await this.started.deployContract({
      abi: ERC1967Proxy.abi,
      bytecode: ERC1967Proxy.bytecode as `0x${string}`,
      salt: "oracleProxy",
      constructorArgs: [
        baseContractAddress,
        encodeFunctionData({
          abi: slayBase.abi,
          functionName: "initialize",
          args: [owner],
        }),
      ],
    });

    // deploy SLAYOracle impl contract
    const oracleImpl = await this.started.deployContract({
      abi: slayOracle.abi,
      bytecode: slayOracle.bytecode.object as `0x${string}`,
      salt: "SLAYOracle",
      constructorArgs: [mockPythContract.contractAddress, this.router.address],
    });

    // get oracleProxy contract instance ( cannot use oracleProxy contract because it is the instance of ERC1967Proxy )
    const oracleProxyContract = getContract({
      address: oracleProxy.contractAddress,
      abi: slayBase.abi,
      client: this.started.getClient(),
    });
    // upgrade oracleProxy to use SLAYOracle impl contract
    await oracleProxyContract.write.upgradeToAndCall([oracleImpl.contractAddress, ""]);

    await this.started.mineBlock(1);

    this.oracle = getContract({
      address: oracleProxy.contractAddress,
      abi: slayOracle.abi,
      client: this.started.getClient(),
    });
  }

  /// Sets oracle price through mockPyth contract
  async setOraclePrice({
    priceId,
    price,
    conf,
    expo,
    timestamp,
  }: {
    priceId: `0x${string}`;
    price: bigint; // price in minor units
    conf: bigint; // confidence in minor units
    expo: number; // minor units decimals (e.g. -8 for 8 decimals)
    timestamp: bigint; // publish timestamp
  }) {
    if (!this.mockPyth) {
      throw new Error("MockPyth contract not initialized. run initOracle first");
    }

    const updateData = await this.mockPyth.read.createPriceFeedUpdateData([
      priceId,
      price,
      conf,
      expo,
      price, // use same price for EMA
      conf, // use same conf for EMA
      timestamp,
      timestamp, // use same timestamp for prevPublishTime
    ]);

    // get update fee ( args updateData is expected in array of bytes, hence the double array )
    const updateFee = await this.mockPyth.read.getUpdateFee([[updateData]]);

    // update price feeds
    await this.mockPyth.write.updatePriceFeeds([[updateData]], { value: updateFee });

    await this.started.mineBlock(1);
  }
}

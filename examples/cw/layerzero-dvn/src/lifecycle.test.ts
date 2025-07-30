import { beforeAll, expect, test, vi } from "vitest";
import {
  AnvilContainer,
  ChainName,
  CosmWasmContract,
  CosmWasmContainer,
  CosmWasmContracts,
  StartedAnvilContainer,
  StartedCosmWasmContainer,
} from "@satlayer/testcontainers";
import simpleOApp from "./evm/out/SimpleOApp.sol/SimpleOApp.json";
import customDVN from "./evm/out/CustomDVN.sol/CustomDVN.json";
import endpointV2 from "@layerzerolabs/lz-evm-protocol-v2/artifacts/contracts/EndpointV2.sol/EndpointV2.json";
import receiveUln302 from "@layerzerolabs/lz-evm-messagelib-v2/artifacts/contracts/uln/uln302/ReceiveUln302.sol/ReceiveUln302.json";
import { LZ_CONTRACTS } from "./lz.constant";
import { encodeFunctionData, getContract, pad, padHex, parseEther } from "viem";
import { Options, PacketV1Codec, trim0x } from "@layerzerolabs/lz-v2-utilities";
import { ConfigHelper, ConfigType } from "./configHelper";
import { DVNNode } from "./dvnNode";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { stringToPath } from "@cosmjs/crypto";
import { readFile } from "node:fs/promises";
import { ExecuteMsg, InstantiateMsg, QueryMsg } from "./bvs-dvn-contract";
import { OperatorNode } from "./operatorNode";

let ethNodeStarted: StartedAnvilContainer;
let opNodeStarted: StartedAnvilContainer;

let cosmosNodeStarted: StartedCosmWasmContainer;
let cosmosClient: SigningCosmWasmClient;
let contracts: CosmWasmContracts;
let wallet: DirectSecp256k1HdWallet;

let ownerAddress: string;
let operator1Address: string;
let operator2Address: string;
let operator3Address: string;
let bvsAddress: string;

let bvsContract: CosmWasmContract<ExecuteMsg, QueryMsg>;

const dvnEthId = 1;
const dvnOpId = 2;

async function deployContract(cosmosClient: SigningCosmWasmClient, owner: string) {
  const contractPath = require.resolve("@examples/layerzero-dvn/src/bvs-contract/dist/contract.wasm");
  const uploaded = await cosmosClient.upload(owner, await readFile(contractPath), "auto");
  const initMsg: InstantiateMsg = {
    registry: contracts.registry.address,
    router: contracts.router.address,
    owner: owner,
    required_verification_threshold: 5000, // 50% of operator have to submit the packet to be finalized
  };
  return cosmosClient.instantiate(owner, uploaded.codeId, initMsg, "dvn-bvs-contract", "auto");
}

// Initializes the BVS contract and registers it with the registry and operators.
async function initBVS(cosmosClient: SigningCosmWasmClient) {
  // deploy BVS contract
  const instantiated = await deployContract(cosmosClient, ownerAddress);
  bvsAddress = instantiated.contractAddress;
  bvsContract = new CosmWasmContract<ExecuteMsg, QueryMsg>(cosmosNodeStarted, bvsAddress);

  // service enable slashing
  await bvsContract.execute(cosmosClient, ownerAddress, {
    enable_slashing: {},
  });

  // register operators to the BVS contract
  await contracts.registry.execute(cosmosClient, operator1Address, {
    register_as_operator: {
      metadata: {},
    },
  });
  await contracts.registry.execute(cosmosClient, operator2Address, {
    register_as_operator: {
      metadata: {},
    },
  });
  await contracts.registry.execute(cosmosClient, operator3Address, {
    register_as_operator: {
      metadata: {},
    },
  });

  // register bvs <-> operator to registry
  await contracts.registry.execute(cosmosClient, operator1Address, {
    register_service_to_operator: {
      service: bvsAddress,
    },
  });
  await contracts.registry.execute(cosmosClient, operator2Address, {
    register_service_to_operator: {
      service: bvsAddress,
    },
  });
  await contracts.registry.execute(cosmosClient, operator3Address, {
    register_service_to_operator: {
      service: bvsAddress,
    },
  });
  await bvsContract.execute(cosmosClient, ownerAddress, {
    register_operator: {
      operator: operator1Address,
    },
  });
  await bvsContract.execute(cosmosClient, ownerAddress, {
    register_operator: {
      operator: operator2Address,
    },
  });
  await bvsContract.execute(cosmosClient, ownerAddress, {
    register_operator: {
      operator: operator3Address,
    },
  });
}

beforeAll(async () => {
  // run local forked nodes
  let ethNode = new AnvilContainer({
    forkedChainName: ChainName.EthereumMainnet,
  });
  let opNode = new AnvilContainer({
    forkedChainName: ChainName.OptimismMainnet,
  });

  ethNodeStarted = await ethNode.start();
  opNodeStarted = await opNode.start();

  // setup local cosmos node with satlayer contracts
  cosmosNodeStarted = await new CosmWasmContainer().start();
  contracts = await CosmWasmContracts.bootstrap(cosmosNodeStarted);

  wallet = await DirectSecp256k1HdWallet.generate(12, {
    prefix: "wasm",
    hdPaths: [stringToPath("m/0"), stringToPath("m/1"), stringToPath("m/2"), stringToPath("m/3")],
  });
  const [owner, operator1, operator2, operator3] = await wallet.getAccounts();

  // Fund all accounts with some tokens
  await cosmosNodeStarted.fund(
    "10000000ustake",
    owner.address,
    operator1.address,
    operator2.address,
    operator3.address,
  );

  ownerAddress = owner.address;
  operator1Address = operator1.address;
  operator2Address = operator2.address;
  operator3Address = operator3.address;

  // Create a new signer with the wallet using the container as the RPC endpoint
  cosmosClient = await cosmosNodeStarted.newSigner(wallet);

  // Initializes the BVS contract and registers it with the registry and operators.
  await initBVS(cosmosClient);
}, 120_000);

test("lifecycle of sending message from ETH -> Optimism using custom DVN + BVS", async () => {
  let ethChainInfo = ethNodeStarted.getChainInfo();
  let opChainInfo = opNodeStarted.getChainInfo();
  let ethLzContracts = LZ_CONTRACTS[ethChainInfo.name];
  let opLzContracts = LZ_CONTRACTS[opChainInfo.name];

  let ethClient = ethNodeStarted.getClient();
  let ethAddress = ethNodeStarted.getAddress();

  let opClient = opNodeStarted.getClient();
  let opAddress = opNodeStarted.getAddress();

  // ========== Deploy DVN and SimpleOApp.sol contracts on ETH and OP chain ==========

  // deploy CustomDVN contract on ETH node
  let deployDvnResEth = await ethNodeStarted.deployContract({
    abi: customDVN.abi,
    bytecode: customDVN.bytecode.object as unknown as `0x${string}`,
    salt: "customdvn",
    constructorArgs: [
      ethLzContracts.eid,
      dvnEthId,
      [ethLzContracts.sendUln302, ethLzContracts.receiveUln302],
      padHex("0x", { size: 20 }),
      [ethAddress],
    ],
  });
  let dvnEthContractAddress = deployDvnResEth.contractAddress;
  let dvnEthContract = deployDvnResEth.contract;

  // deploy CustomDVN contract on Optimism node
  let deployDvnResOp = await opNodeStarted.deployContract({
    abi: customDVN.abi,
    bytecode: customDVN.bytecode.object as unknown as `0x${string}`,
    salt: "customdvn",
    constructorArgs: [
      opLzContracts.eid,
      dvnOpId,
      [opLzContracts.sendUln302, opLzContracts.receiveUln302],
      padHex("0x", { size: 20 }),
      [opAddress],
    ],
  });
  let dvnOpContractAddress = deployDvnResOp.contractAddress;
  let dvnOpContract = deployDvnResOp.contract;

  // fund dvn contract on Optimism node
  await opNodeStarted.setBalance(dvnOpContractAddress, parseEther("100", "wei"));

  // deploy SimpleOApp.sol on ETH node
  let deployMyOAppResEth = await ethNodeStarted.deployContract({
    abi: simpleOApp.abi,
    bytecode: simpleOApp.bytecode.object as unknown as `0x${string}`,
    salt: "myoapp",
    constructorArgs: [ethLzContracts.endpointV2, ethAddress],
  });
  let oappEthContractAddress = deployMyOAppResEth.contractAddress;
  let oappEthContract = deployMyOAppResEth.contract;

  // deploy SimpleOApp.sol on Optimism node
  let deployMyOAppResOp = await opNodeStarted.deployContract({
    abi: simpleOApp.abi,
    bytecode: simpleOApp.bytecode.object as unknown as `0x${string}`,
    salt: "myoapp",
    constructorArgs: [opLzContracts.endpointV2, opAddress],
  });
  let oappOpContractAddress = deployMyOAppResOp.contractAddress;
  let oappOpContract = deployMyOAppResOp.contract;

  // assert OP MyOapp data is empty
  let myOappOpData = await oappOpContract.read.data([]);
  expect(myOappOpData).toStrictEqual("Nothing received yet");

  // ========== Setup Off-chain nodes| DVNNode + OperatorNode ==========
  // setup DVN node to listen to jobs assigned by LZ in eth mainnet and submit them to BVS contract
  let dvnNode = new DVNNode({
    cosmosClient,
    ethClient,
    opClient,
    ownerAddress,
    bvsAddress,
    dvnEthContractAddress,
  });
  let currentEthBlock = await ethClient.getBlockNumber();
  void dvnNode.startEthNode(currentEthBlock);
  {
    // setup Operator nodes to listen to packets and submit them to BVS
    let operator1Node = new OperatorNode({
      client: cosmosClient,
      bvsAddress,
      operatorAddress: operator1Address,
      label: "Operator1Node",
    });
    let operator2Node = new OperatorNode({
      client: cosmosClient,
      bvsAddress,
      operatorAddress: operator2Address,
      label: "Operator2Node",
    });
    let operator3Node = new OperatorNode({
      client: cosmosClient,
      bvsAddress,
      operatorAddress: operator3Address,
      label: "Operator3Node",
    });
    void operator1Node.start(0);
    void operator2Node.start(0);
    void operator3Node.start(0);
  }

  // ========== Setup custom DVN routing for SimpleOApp.sol for ETH -> OP ==========

  // SET PEERs
  await oappEthContract.write.setPeer([opLzContracts.eid, pad(oappOpContractAddress)]);
  await oappOpContract.write.setPeer([ethLzContracts.eid, pad(oappEthContractAddress)]);

  // set Send and Receive lib for SimpleOApp.sol on ETH and Optimism
  let endpointV2ContractEth = getContract({
    address: ethLzContracts.endpointV2 as `0x${string}`,
    abi: endpointV2.abi,
    client: ethClient,
  });
  await endpointV2ContractEth.write.setSendLibrary([
    oappEthContractAddress,
    opLzContracts.eid,
    ethLzContracts.sendUln302,
  ]);
  await endpointV2ContractEth.write.setReceiveLibrary([
    oappEthContractAddress,
    opLzContracts.eid,
    ethLzContracts.receiveUln302,
    0,
  ]);

  let endpointV2ContractOp = getContract({
    address: opLzContracts.endpointV2 as `0x${string}`,
    abi: endpointV2.abi,
    client: opClient,
  });
  await endpointV2ContractOp.write.setSendLibrary([
    oappOpContractAddress,
    ethLzContracts.eid,
    opLzContracts.sendUln302,
  ]);
  await endpointV2ContractOp.write.setReceiveLibrary([
    oappOpContractAddress,
    ethLzContracts.eid,
    opLzContracts.receiveUln302,
    0,
  ]);

  // create DVN route config for SimpleOApp.sol for ETH -> Optimism
  // If not set, the default DVNs will be used.
  // also there must be at least one optional DVN configured.
  let fromEthToOpConfig = ConfigHelper.from([
    {
      eid: opLzContracts.eid,
      configType: ConfigType.EXECUTOR,
      config: {
        maxMessageSize: 10000, // max message size in bytes
        executor: ethLzContracts.LZExecutor, // executor address
      },
    },
    {
      eid: opLzContracts.eid,
      configType: ConfigType.ULN,
      config: {
        confirmations: BigInt(0), // null means no confirmations are required, 0 means default confirmations are used.
        requiredDVNs: [dvnEthContractAddress],
        optionalDVNs: [],
        optionalDVNThreshold: 0,
      },
    },
  ]);

  // set ETH sendlib config (for ETH -> Optimism)
  await endpointV2ContractEth.write.setConfig([
    oappEthContractAddress,
    ethLzContracts.sendUln302,
    fromEthToOpConfig.toConfigParams(),
  ]);

  let fromOpToEthConfig = ConfigHelper.from([
    {
      eid: ethLzContracts.eid,
      configType: ConfigType.ULN,
      config: {
        confirmations: BigInt(0), // null means no confirmations are required, 0 means default confirmations are used.
        requiredDVNs: [dvnOpContractAddress],
        optionalDVNs: [],
        optionalDVNThreshold: 0,
      },
    },
  ]);
  // set Optimism receiveLib config (for ETH -> Optimism)
  await endpointV2ContractOp.write.setConfig([
    oappOpContractAddress,
    opLzContracts.receiveUln302,
    fromOpToEthConfig.toConfigParams(),
  ]);

  // move the blocks to ensure the configuration is set
  await opNodeStarted.mineBlock();
  await ethNodeStarted.mineBlock();

  // ========== Send data from SimpleOApp.sol ETH -> SimpleOApp.sol Optimism using custom DVN + BVS ==========

  // send data from ETH -> Optimism
  let sendConfig = Options.newOptions().addExecutorLzReceiveOption(50000, 0);
  let quoteRes = await oappEthContract.simulate.quote([opLzContracts.eid, "Eth says hello!", sendConfig.toHex()]);
  let nativeFee: bigint = quoteRes.result[0];

  const { request } = await oappEthContract.simulate.send([opLzContracts.eid, "Eth says hello!", sendConfig.toHex()], {
    value: nativeFee,
  });
  await oappEthContract.write.send(request);

  // mine the tx
  await ethNodeStarted.mineBlock();

  // get tx events
  let resEvents = await endpointV2ContractEth.getEvents.PacketSent();

  // get encoded packet.
  // @ts-ignore
  let encodedPacket = resEvents[0].args.encodedPayload;
  let packet = PacketV1Codec.from(encodedPacket);

  // ========== Wait for BVS operator to verify packets ==========

  // BVS checks if threshold no. of operators (2 of 3) have submitted the packet
  await vi.waitFor(
    async () => {
      let finalizePacketVerificationRes = await dvnNode.finalizedPacket(trim0x(packet.guid()));
      expect(finalizePacketVerificationRes).toBeDefined();
    },
    {
      interval: 1000,
      timeout: 60_000,
    },
  );

  // Get the finalized packet from BVS
  let finalizedPacketPayloadHash = await dvnNode.getFinalizedPayloadHash(trim0x(packet.guid()));
  expect(finalizedPacketPayloadHash).toBeDefined();

  // TODO: get receive lib to get confirmation required before calling `verify`

  // ========== Confirm packet verification on destination chain (OP) ==========

  // call verify on receivelib on Optimism through DVN contract to verify the packet
  let verifyCallData = encodeFunctionData({
    abi: receiveUln302.abi,
    functionName: "verify",
    args: [packet.header() as `0x${string}`, `0x${finalizedPacketPayloadHash}` as `0x${string}`, BigInt(100)],
  });
  let currentBlock = await opClient.getBlock();
  let expiration = BigInt(currentBlock.timestamp + BigInt(60 * 60 * 1000));
  // get signature of dvn for verifycalldata
  let calldataHashRes = await dvnOpContract.simulate.hashCallData([
    2,
    opLzContracts.receiveUln302,
    verifyCallData,
    expiration,
  ]);
  let calldataHash = calldataHashRes.result;
  let signerAccount = opNodeStarted.getAccount();
  let dvnSignature = await signerAccount.signMessage!({ message: calldataHash });

  // call verify on OP DVN contract
  await dvnOpContract.write.execute([
    [
      {
        vid: dvnOpId, // OP DVN ID
        target: opLzContracts.receiveUln302,
        callData: verifyCallData,
        expiration: expiration,
        signatures: dvnSignature,
      },
    ],
  ]);

  await opNodeStarted.mineBlock();

  // call commitVerification on Optimism receivelib (can be called by anyone)
  let receiveLibContract = getContract({
    address: opLzContracts.receiveUln302 as `0x${string}`,
    abi: receiveUln302.abi,
    client: opClient,
  });
  await receiveLibContract.write.commitVerification([
    packet.header() as `0x${string}`,
    packet.payloadHash() as `0x${string}`,
  ]);

  // call lzReceive on Optimism OApp contract (typically called by the executor)
  await endpointV2ContractOp.write.lzReceive([
    {
      srcEid: ethLzContracts.eid, // ETH EID
      sender: pad(oappEthContractAddress),
      nonce: packet.nonce(),
    },
    oappOpContractAddress,
    packet.guid(),
    packet.message(),
    "",
  ]);

  await opNodeStarted.mineBlock();

  {
    // check for dvn execute failed event
    let dvnExecuteFailedEvents = await dvnOpContract.getEvents.ExecuteFailed();
    expect(dvnExecuteFailedEvents.length).toBe(0);
  }

  // TODO: do final idempotency check on DVN contract to see if packet is successfully verified

  // expect OP OApp data is updated
  let opOAppData = await oappOpContract.read.data([]);
  expect(opOAppData).toStrictEqual("Eth says hello!");
}, 120_000);

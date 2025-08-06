import { AbstractStartedContainer, GenericContainer, StartedTestContainer, Wait } from "testcontainers";
import {
  Abi,
  Account,
  Chain,
  Client,
  createTestClient,
  encodeDeployData,
  getContract,
  http,
  isHex,
  keccak256,
  parseEther,
  Prettify,
  PublicActions,
  publicActions,
  RpcSchema,
  TestActions,
  TestRpcSchema,
  toHex,
  Transport,
  WalletActions,
  walletActions,
} from "viem";
import { generatePrivateKey, mnemonicToAccount, privateKeyToAccount } from "viem/accounts";
import { mainnet, optimism } from "viem/chains";

export type ChainInfo = {
  name: ChainName;
  chainId: number;
  rpcEndpoint: string;
  forkBlockNumber: number;
  viemChain: Chain;
};

export enum ChainName {
  EthereumMainnet = "ethereum-mainnet",
  OptimismMainnet = "optimism-mainnet",
}

/**
 * Maps chain names to their respective chain information.
 * Update this object to add more chains as needed.
 *
 * `forkBlockNumber` is an arbitrary value.
 */
const NAME_TO_CHAIN_INFO: Record<ChainName, ChainInfo> = {
  [ChainName.EthereumMainnet]: {
    name: ChainName.EthereumMainnet,
    chainId: 1,
    rpcEndpoint: "https://gateway.tenderly.co/public/mainnet",
    forkBlockNumber: 22637910,
    viemChain: mainnet,
  },
  [ChainName.OptimismMainnet]: {
    name: ChainName.OptimismMainnet,
    chainId: 10,
    rpcEndpoint: "https://mainnet.optimism.io",
    forkBlockNumber: 136800904,
    viemChain: optimism,
  },
};

const MNEMONIC = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";

// from https://github.com/thegostep/solidity-create2-deployer/blob/6b7e8cf294042ce0b6575f81aa14f223154c482f/src/utils.ts#L7
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const CREATE2_BYTECODE: `0x${string}` =
  "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf3";

// from https://github.com/thegostep/solidity-create2-deployer/blob/6b7e8cf294042ce0b6575f81aa14f223154c482f/src/utils.ts#L5
const CREATE2_DEPLOYER_ADDRESS: `0x${string}` = "0x4a27c059FD7E383854Ea7DE6Be9c390a795f6eE3";

// from https://github.com/thegostep/solidity-create2-deployer/blob/6b7e8cf294042ce0b6575f81aa14f223154c482f/src/utils.ts#L8-L48
const CREATE2_ABI = [
  {
    anonymous: false,
    inputs: [
      {
        indexed: false,
        internalType: "address",
        name: "addr",
        type: "address",
      },
      {
        indexed: false,
        internalType: "uint256",
        name: "salt",
        type: "uint256",
      },
    ],
    name: "Deployed",
    type: "event",
  },
  {
    constant: false,
    inputs: [
      {
        internalType: "bytes",
        name: "code",
        type: "bytes",
      },
      {
        internalType: "uint256",
        name: "salt",
        type: "uint256",
      },
    ],
    name: "deploy",
    outputs: [],
    payable: false,
    stateMutability: "nonpayable",
    type: "function",
  },
];

export class AnvilContainer extends GenericContainer {
  private readonly anvilPort = 8545; // default anvil port
  private readonly chainId = 31337; // default anvil chain id
  private forkedChainName = ChainName.EthereumMainnet;

  constructor({ forkedChainName = ChainName.EthereumMainnet }: { forkedChainName: ChainName }) {
    const image = "ghcr.io/foundry-rs/foundry:latest";
    super(image);

    const chainInfo = AnvilContainer.getChainInfoByName(forkedChainName);
    this.forkedChainName = chainInfo.name;

    this.withExposedPorts(this.anvilPort);
    this.withCommand([
      `anvil --fork-url ${chainInfo.rpcEndpoint} --fork-block-number ${chainInfo.forkBlockNumber} --fork-chain-id ${chainInfo.chainId} --accounts 0 --no-mining --auto-impersonate`,
    ]);
    this.withEnvironment({
      ANVIL_IP_ADDR: "0.0.0.0",
    });
    this.withWaitStrategy(Wait.forLogMessage(/Listening on/));
  }

  get ChainId(): number {
    return this.chainId;
  }

  public override async start(): Promise<StartedAnvilContainer> {
    const started = await super.start();

    return new StartedAnvilContainer(started, this.forkedChainName, this.anvilPort);
  }

  static getChainIdByName(name: ChainName): number {
    return NAME_TO_CHAIN_INFO[name].chainId;
  }

  static getChainInfoByName(name: ChainName): ChainInfo {
    return NAME_TO_CHAIN_INFO[name];
  }
}

// from https://github.com/wevm/viem/blob/2bfdda8ed120c3d1fb2c10192d31538e7cd601cf/src/clients/createTestClient.ts#L18
type TestClientMode = "anvil" | "hardhat" | "ganache";

// TestClient + PublicActions + WalletActions (if includeActions is true) currently not supported by viem
export type SuperTestClient<
  mode extends TestClientMode = TestClientMode,
  transport extends Transport = Transport,
  chain extends Chain | undefined = Chain | undefined,
  account extends Account | undefined = Account | undefined,
  rpcSchema extends RpcSchema | undefined = undefined,
> = Prettify<
  { mode: mode } & Client<
    transport,
    chain,
    account,
    rpcSchema extends RpcSchema ? [...TestRpcSchema<mode>, ...rpcSchema] : TestRpcSchema<mode>,
    TestActions & PublicActions<transport, chain, account> & WalletActions<chain, account>
  >
>;

export class StartedAnvilContainer extends AbstractStartedContainer {
  private readonly chainName: ChainName;
  private readonly port: number;
  private readonly client: SuperTestClient<"anvil", Transport, Chain, Account, RpcSchema>;
  private readonly account: Account;

  constructor(startedTestContainer: StartedTestContainer, chainName: ChainName, port = 8545) {
    super(startedTestContainer);
    const chainInfo = AnvilContainer.getChainInfoByName(chainName);
    this.chainName = chainName;
    this.port = port;

    this.account = mnemonicToAccount(MNEMONIC);

    this.client = createTestClient({
      chain: chainInfo.viemChain,
      mode: "anvil",
      account: this.account,
      transport: http(this.getRpcEndpoint()),
      pollingInterval: 100, // 100ms polling interval
      cacheTime: 0, // disable caching
    })
      .extend(publicActions)
      .extend(walletActions);

    // fund account with 100 ETH
    void this.setBalance(this.account.address, parseEther("100", "wei"));
  }

  getRpcEndpoint(): string {
    const host = this.getHost();
    const port = this.getMappedPort(this.port);
    return `http://${host}:${port}`;
  }

  getRpcPort(): number {
    return this.port;
  }

  getChainInfo() {
    return AnvilContainer.getChainInfoByName(this.chainName);
  }

  getClient(): SuperTestClient {
    return this.client;
  }

  async mineBlock(n: number = 1): Promise<void> {
    return this.client.mine({ blocks: n, interval: n }); // move block by n blocks and n seconds
  }

  getAddress(): `0x${string}` {
    return this.account.address;
  }

  getAccount(): Account {
    return this.account;
  }

  getRandomAccount(): Account {
    const randomPrivKey = generatePrivateKey();
    return privateKeyToAccount(randomPrivKey);
  }

  generateAccount(salt: string): Account {
    const privateKey = keccak256(toHex(salt));
    return privateKeyToAccount(privateKey as `0x${string}`);
  }

  getRandomAddress(): `0x${string}` {
    const randomAccount = this.getRandomAccount();
    return randomAccount.address as `0x${string}`;
  }

  setBalance(address: `0x${string}`, amount: bigint) {
    return this.client.setBalance({ address, value: amount });
  }

  /**
   * Deploys a contract using CREATE2 deployer in `CREATE2_DEPLOYER_ADDRESS`,
   * which should be available in most EVM compatible chain.
   *
   * This method uses the `CREATE2` opcode to deploy a contract at a deterministic address.
   *
   * @param abi
   * @param bytecode
   * @param salt
   * @param constructorArgs
   */
  async deployContract({
    abi,
    bytecode,
    salt,
    constructorArgs,
  }: {
    abi: Abi | unknown[];
    bytecode: `0x${string}`;
    salt: string;
    constructorArgs: any[];
  }) {
    const create2Contract = getContract({
      address: CREATE2_DEPLOYER_ADDRESS,
      abi: CREATE2_ABI,
      client: this.client,
    });

    const saltHex = saltToHex(salt);

    const deployData = encodeDeployData({ abi, args: constructorArgs, bytecode });

    // deploy
    const deployTx = await create2Contract.write.deploy([deployData, saltHex]);

    // mine a block to ensure the transaction is processed
    await this.mineBlock();

    // get tx receipt
    const txReceipt = await this.client.getTransactionReceipt({ hash: deployTx });

    // get logs
    const log = await create2Contract.getEvents.Deployed();
    // @ts-expect-error arguments is not typed in viem
    const deployedAddress = log[0].args.addr as `0x${string}`;

    const computedAddress = StartedAnvilContainer.getCreate2Address({ deployBytecode: deployData, saltHex });

    if (deployedAddress.toLowerCase() !== computedAddress.toLowerCase()) {
      throw new Error(
        `Deployed contract address ${deployedAddress} does not match computed address ${computedAddress}`,
      );
    }

    return {
      txHash: deployTx,
      contractAddress: deployedAddress as `0x${string}`,
      contract: getContract({
        address: deployedAddress,
        abi: abi,
        client: this.client,
      }),
      txReceipt: txReceipt,
    };
  }

  static getCreate2Address({
    deployBytecode,
    saltHex,
  }: {
    deployBytecode: `0x${string}`;
    saltHex: `0x${string}`;
  }): `0x${string}` {
    return `0x${keccak256(`0xff${trim0x(CREATE2_DEPLOYER_ADDRESS)}${trim0x(saltHex)}${trim0x(keccak256(deployBytecode))}`).slice(-40)}` as `0x${string}`;
  }
}

const trim0x = (hex: `0x${string}`): string => {
  if (hex.startsWith("0x")) {
    return hex.slice(2);
  }
  return hex;
};

export const saltToHex = (salt: string): `0x${string}` => {
  if (isHex(salt)) {
    return salt as `0x${string}`;
  }
  return keccak256(toHex(salt));
};

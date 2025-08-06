import { parseEther } from "viem";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { AnvilContainer, ChainName, saltToHex, StartedAnvilContainer } from "./anvil-container";

let started: StartedAnvilContainer;

beforeAll(async () => {
  // Use Ethereum mainnet for testing
  started = await new AnvilContainer({
    forkedChainName: ChainName.EthereumMainnet,
  }).start();
}, 60_000); // Allow up to 60 seconds for container startup

afterAll(async () => {
  await started.stop();
});

describe("AnvilContainer", () => {
  it("should start and provide an RPC endpoint", () => {
    const rpcEndpoint = started.getRpcEndpoint();
    expect(rpcEndpoint).toMatch(/^http:\/\/.*:\d+$/);
  });

  it("should provide a client for interacting with the blockchain", () => {
    const client = started.getClient();
    expect(client).toBeDefined();
    expect(client.mode).toBe("anvil");
  });

  it("should have a funded account", async () => {
    const address = started.getAddress();
    const client = started.getClient();
    const balance = await client.getBalance({ address });
    expect(balance).toBeGreaterThanOrEqual(parseEther("100", "wei"));
  });

  it("should have a working client with mining capabilities", async () => {
    const client = started.getClient();

    // Verify the client has the expected mode
    expect(client.mode).toBe("anvil");

    // Verify we can get the current block number
    const blockNumber = await client.getBlockNumber();
    expect(typeof blockNumber).toBe("bigint");

    // The mine method exists on the client
    expect(typeof client.mine).toBe("function");

    // Note: In Anvil with --no-mining flag, blocks are only mined when transactions occur
    // So we'll verify mining indirectly by performing a transaction
    const testAddress = started.getRandomAddress();
    const amount = parseEther("5", "wei");

    // send some ether to the test address
    // @ts-expect-error viem's sendTransaction type error
    await client.sendTransaction({
      to: testAddress,
      value: amount,
    });

    // check that test address has not received the funds yet
    let balance = await client.getBalance({ address: testAddress });
    expect(balance).toBe(BigInt(0));

    // mine the transaction
    await started.mineBlock();

    // check that test address has received the funds
    balance = await client.getBalance({ address: testAddress });
    expect(balance).toBe(amount);
  });

  it("should mine block and advance timestamp proportionally", async () => {
    const client = started.getClient();

    // Get the current block number and timestamp
    const initialBlock = await client.getBlock();
    const initialTimestamp = initialBlock.timestamp;

    // Mine 5 blocks
    await started.mineBlock(5);

    // Get the new block number and timestamp
    const newBlock = await client.getBlock();
    const newTimestamp = newBlock.timestamp;

    // Verify that the block number has increased by 5
    expect(newBlock.number).toStrictEqual(initialBlock.number + BigInt(5));

    // Verify that the timestamp has increased proportionally
    expect(newTimestamp).toBeGreaterThanOrEqual(initialTimestamp + BigInt(5));
  });

  it("should be able to set balance for an address", async () => {
    const testAddress = "0x1234567890123456789012345678901234567890";
    const amount = parseEther("10", "wei");

    await started.setBalance(testAddress, amount);

    const client = started.getClient();
    const balance = await client.getBalance({ address: testAddress });
    expect(balance).toBe(amount);
  });

  it("should be able to deploy a contract with create2", async () => {
    // Simple counter contract bytecode
    const bytecode =
      "0x6080604052348015600e575f5ffd5b506101e18061001c5f395ff3fe608060405234801561000f575f5ffd5b506004361061003f575f3560e01c80633fb5c1cb146100435780638381f58a1461005f578063d09de08a1461007d575b5f5ffd5b61005d600480360381019061005891906100e4565b610087565b005b610067610090565b604051610074919061011e565b60405180910390f35b610085610095565b005b805f8190555050565b5f5481565b5f5f8154809291906100a690610164565b9190505550565b5f5ffd5b5f819050919050565b6100c3816100b1565b81146100cd575f5ffd5b50565b5f813590506100de816100ba565b92915050565b5f602082840312156100f9576100f86100ad565b5b5f610106848285016100d0565b91505092915050565b610118816100b1565b82525050565b5f6020820190506101315f83018461010f565b92915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f61016e826100b1565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82036101a05761019f610137565b5b60018201905091905056fea264697066735822122024f0977bbb419369959b0d25e457f944630262e429526da2c6d6e0d48b596f2d64736f6c634300081e0033";

    const abi = [
      {
        inputs: [],
        name: "increment",
        outputs: [],
        stateMutability: "nonpayable",
        type: "function",
      },
      {
        inputs: [],
        name: "number",
        outputs: [
          {
            internalType: "uint256",
            name: "",
            type: "uint256",
          },
        ],
        stateMutability: "view",
        type: "function",
      },
      {
        inputs: [
          {
            internalType: "uint256",
            name: "newNumber",
            type: "uint256",
          },
        ],
        name: "setNumber",
        outputs: [],
        stateMutability: "nonpayable",
        type: "function",
      },
    ];

    const predictedAddress = StartedAnvilContainer.getCreate2Address({
      deployBytecode: bytecode,
      saltHex: saltToHex("counter"),
    });

    const result = await started.deployContract({
      abi,
      bytecode,
      salt: "counter",
      constructorArgs: [],
    });

    // Verify deployment was successful
    expect(result.contractAddress).toBeDefined();
    expect(result.txHash).toBeDefined();
    expect(result.contractAddress.toLowerCase()).toStrictEqual(predictedAddress.toLowerCase());

    // Verify contract code exists at the deployed address
    const client = started.getClient();
    const code = await client.getCode({ address: result.contractAddress });
    expect(code).toBeTruthy();
    expect(code?.length).toBeGreaterThan(2); // More than just '0x'
  });
});

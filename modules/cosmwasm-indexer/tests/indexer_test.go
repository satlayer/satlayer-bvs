package tests

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/cosmos/cosmos-sdk/types"
	dbconfig "github.com/forbole/juno/v6/database/config"
	loggingconfig "github.com/forbole/juno/v6/logging/config"
	nodeconfig "github.com/forbole/juno/v6/node/config"
	noderemote "github.com/forbole/juno/v6/node/remote"
	parserconfig "github.com/forbole/juno/v6/parser/config"
	junoconfig "github.com/forbole/juno/v6/types/config"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/types/config"
	"github.com/stretchr/testify/suite"
	"gopkg.in/yaml.v3"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/cw20"
	api "github.com/satlayer/satlayer-bvs/cosmwasm-api"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/modules/wasm"
	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/wait"
)

type CosmWasmIndexerTestSuite struct {
	suite.Suite
	babylonContainer *babylond.BabylonContainer
	dbContainer      *PostgreSQLContainer
	indexerContainer testcontainers.Container
	contract         *babylond.DeployedWasmContract
	userA            types.AccAddress
	userB            types.AccAddress
}

func (c *CosmWasmIndexerTestSuite) generateYAMLConfig() error {
	junoCfg := junoconfig.Config{
		Chain: junoconfig.ChainConfig{
			Bech32Prefix: "bbn",
			Modules:      []string{"wasm"},
		},
		Node: nodeconfig.Config{
			Type: "remote",
			Details: &noderemote.Details{
				RPC: &noderemote.RPCConfig{
					ClientName:     "babylon",
					Address:        c.babylonContainer.RpcUri,
					MaxConnections: 10,
				},
				GRPC: &noderemote.GRPCConfig{
					Address:  c.babylonContainer.GrpcUri,
					Insecure: true,
				},
				API: &noderemote.APIConfig{Address: c.babylonContainer.ApiUri},
			},
		},
		Parser: parserconfig.DefaultParsingConfig(),
		Database: dbconfig.Config{
			URL:                c.dbContainer.URL,
			MaxOpenConnections: 1,
			MaxIdleConnections: 1,
			PartitionSize:      100000,
			PartitionBatchSize: 1000,
			SSLModeEnable:      "false",
			SSLRootCert:        "",
			SSLCert:            "",
			SSLKey:             "",
		},
		Logging: loggingconfig.DefaultLoggingConfig(),
	}

	wasmConfig := wasm.Config{
		Contracts: map[string]string{c.contract.Address: "cw20"},
		CodeIDs:   []uint64{c.contract.CodeId},
	}

	indexerConfig := config.Config{
		JunoConfig: junoCfg,
		WasmConfig: wasmConfig,
	}

	yamlData, err := yaml.Marshal(indexerConfig)
	if err != nil {
		return fmt.Errorf("failed to marshal config to YAML: %w", err)
	}

	configPath := filepath.Join("./testdata", "config.yaml")
	if err = os.WriteFile(configPath, yamlData, 0644); err != nil {
		return fmt.Errorf("failed to write config file: %w", err)
	}

	return nil
}

func (c *CosmWasmIndexerTestSuite) RunIndexer() error {
	ctx := context.Background()

	if err := c.generateYAMLConfig(); err != nil {
		return fmt.Errorf("failed to generate config: %w", err)
	}

	req := testcontainers.ContainerRequest{
		Image: "golang:1.24.2-alpine3.20",
		Env: map[string]string{
			"GOPATH":  "/go",
			"GOCACHE": "/go/cache",
		},
		Mounts: testcontainers.ContainerMounts{
			testcontainers.ContainerMount{
				Source: testcontainers.GenericBindMountSource{
					HostPath: ".",
				},
				Target: "/app",
			},
		},
		Cmd: []string{
			"sh", "-c",
			"cd /app && go build -o indexer ./cmd/indexer && ./indexer start --home ./testdata/",
		},
		ExposedPorts: []string{"8080/tcp"},
		WaitingFor: wait.ForLog("Starting indexer").
			WithStartupTimeout(30 * time.Second),
	}

	container, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
		ContainerRequest: req,
		Started:          true,
	})
	if err != nil {
		return fmt.Errorf("failed to start container: %w", err)
	}

	host, err := container.Host(ctx)
	if err != nil {
		return fmt.Errorf("failed to get container host: %w", err)
	}

	port, err := container.MappedPort(ctx, "8080")
	if err != nil {
		return fmt.Errorf("failed to get container port: %w", err)
	}

	url := fmt.Sprintf("http://%s:%s", host, port.Port())
	fmt.Println("Indexer url: ", url)

	// if err := wait.ForHTTP("/health").
	// 	WithStartupTimeout(30*time.Second).
	// 	WaitUntilReady(ctx, container); err != nil {
	// 	return fmt.Errorf("failed to wait for service: %w", err)
	// }

	c.indexerContainer = container

	return nil
}

func (c *CosmWasmIndexerTestSuite) SetupSuite() {
	c.babylonContainer = babylond.Run(context.Background())
	c.dbContainer = Run(context.Background())
	c.DeployCw20Token()
}

func (c *CosmWasmIndexerTestSuite) TearDownSuite() {
	ctx := context.Background()
	if c.indexerContainer != nil {
		c.Require().NoError(c.indexerContainer.Terminate(ctx))
	}
	c.Require().NoError(c.babylonContainer.Terminate(ctx))
	c.Require().NoError(c.dbContainer.Terminate(ctx))
}

func (c *CosmWasmIndexerTestSuite) DeployCw20Token() {
	c.userA = c.babylonContainer.GenerateAddress("userA")
	initMsg := cw20.InstantiateMsg{
		Decimals: 6,
		InitialBalances: []cw20.Cw20Coin{
			{
				Address: c.userA.String(),
				Amount:  "1000000000",
			},
		},
		Mint: &cw20.MinterResponse{
			Minter: c.userA.String(),
		},
		Name:   "Test Token",
		Symbol: "TEST",
	}

	contract := cw20.DeployCw20(c.babylonContainer, initMsg)
	c.NotEmpty(contract.Address)

	c.contract = contract
}

func (c *CosmWasmIndexerTestSuite) ExecuteMint() {
	clientCtx := api.NewClientCtx(c.babylonContainer.RpcUri, c.babylonContainer.ChainId).
		WithKeyring(c.babylonContainer.ClientCtx.Keyring).
		WithFromAddress(c.userA).
		WithFromName("userA")

	c.userB = c.babylonContainer.GenerateAddress("userB")
	executeMsg := cw20.ExecuteMsg{Mint: &cw20.Mint{
		Amount:    "100",
		Recipient: c.userB.String(),
	}}

	executeOptions := api.DefaultBroadcastOptions().
		WithContractAddr(c.contract.Address).
		WithExecuteMsg(executeMsg).
		WithGasPrice("0.002ubbn")

	response, err := api.Execute(
		clientCtx,
		context.Background(),
		c.userA.String(),
		executeOptions,
	)
	fmt.Println("Mint method response: ", response)

	c.Require().NoError(err)
}

func TestCosmWasmIndexer(t *testing.T) {
	suite.Run(t, new(CosmWasmIndexerTestSuite))
}

package tests

import (
	"context"
	"database/sql"
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
	"github.com/jmoiron/sqlx"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"
	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/wait"
	"gopkg.in/yaml.v3"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/cw20"
	api "github.com/satlayer/satlayer-bvs/cosmwasm-api"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/modules/wasm"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/types/config"
)

type CosmWasmIndexerTestSuite struct {
	suite.Suite
	babylonContainer *babylond.BabylonContainer
	dbContainer      *PostgreSQLContainer
	indexerContainer testcontainers.Container

	contract *babylond.DeployedWasmContract
	userA    types.AccAddress
	userB    types.AccAddress
}

func (c *CosmWasmIndexerTestSuite) RunIndexer(ctx context.Context) error {
	projectRoot, err := filepath.Abs("../../..")
	if err != nil {
		return fmt.Errorf("failed to generate config: %w", err)
	}

	if err = c.generateYAMLConfig(); err != nil {
		return fmt.Errorf("failed to generate config: %w", err)
	}

	req := testcontainers.ContainerRequest{
		Image: "golang:1.24.2-alpine3.20",
		Env: map[string]string{
			"GOPATH":      "/go",
			"GOCACHE":     "/go/cache",
			"GO111MODULE": "on",
		},
		Mounts: testcontainers.ContainerMounts{
			testcontainers.ContainerMount{
				Source: testcontainers.GenericBindMountSource{
					HostPath: projectRoot,
				},
				Target: "/app",
			},
		},
		Cmd: []string{
			"sh", "-c",
			"cd /app/cosmwasm-indexer && " +
				"go build -v -o indexer ./cmd/indexer && " +
				"echo 'Build completed' && " +
				"./indexer start --home /app/cosmwasm-indexer/tests/e2e/testdata",
		},
		WaitingFor: wait.ForAll(
			wait.ForLog("Build completed").WithStartupTimeout(200*time.Second),
			wait.ForExec([]string{"pgrep", "indexer"}).
				WithStartupTimeout(10*time.Second).
				WithPollInterval(2*time.Second),
		),
	}

	indexerContainer, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
		ContainerRequest: req,
		Started:          true,
	})
	if err != nil {
		return fmt.Errorf("failed to start running indexer container: %w", err)
	}

	c.indexerContainer = indexerContainer
	return nil
}

// Container running order: Babylon chain -> PostgreSQL -> Indexer
func (c *CosmWasmIndexerTestSuite) SetupSuite() {
	ctx := context.Background()

	c.babylonContainer = babylond.Run(ctx)
	c.dbContainer = Run(ctx)

	c.Cw20Contract()

	if err := c.RunIndexer(ctx); err != nil {
		panic(err)
	}
}

func (c *CosmWasmIndexerTestSuite) TearDownSuite() {
	ctx := context.Background()

	if c.indexerContainer != nil {
		c.Require().NoError(c.indexerContainer.Terminate(ctx))
	}

	if c.babylonContainer != nil {
		c.Require().NoError(c.babylonContainer.Terminate(ctx))
	}

	if c.dbContainer != nil {
		c.Require().NoError(c.dbContainer.Terminate(ctx))
	}
}

func (c *CosmWasmIndexerTestSuite) Cw20Contract() {
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

	c.executeMint()
}

func (c *CosmWasmIndexerTestSuite) executeMint() {
	_ = c.babylonContainer.FundAddressUbbn(c.userA.String(), 1000)
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

	_, err := api.Execute(
		clientCtx,
		context.Background(),
		c.userA.String(),
		executeOptions,
	)

	c.Require().NoError(err)
}

func TestCosmWasmIndexer(t *testing.T) {
	suite.Run(t, new(CosmWasmIndexerTestSuite))
}

func (c *CosmWasmIndexerTestSuite) TestIndexer() {
	time.Sleep(5 * time.Second)

	db, err := sql.Open("postgres", c.dbContainer.URL)
	require.NoError(c.T(), err)
	defer db.Close()

	sqlxDB := sqlx.NewDb(db, "postgresql")
	defer sqlxDB.Close()

	var count int
	err = sqlxDB.Get(&count, `SELECT count(*) FROM wasm_execute_contract`)
	require.NoError(c.T(), err)
	require.Equal(c.T(), count, 1)
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
					Address:        c.babylonContainer.SubnetRPCURI,
					MaxConnections: 10,
				},
				GRPC: &noderemote.GRPCConfig{
					Address:  c.babylonContainer.SubnetGRPCURI,
					Insecure: true,
				},
				API: &noderemote.APIConfig{Address: c.babylonContainer.SubnetAPIURI},
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

	currentPath, err := filepath.Abs(".")
	if err != nil {
		return fmt.Errorf("failed to get current root path: %w", err)
	}

	configPath := filepath.Join(currentPath, "testdata/config.yaml")
	if err = os.WriteFile(configPath, yamlData, 0644); err != nil {
		return fmt.Errorf("failed to write config file: %w", err)
	}

	return nil
}

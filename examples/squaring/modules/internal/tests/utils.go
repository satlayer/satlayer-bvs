package tests

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"

	squaringcontract "github.com/satlayer/satlayer-bvs/examples/squaring/squaring-contract"

	"github.com/satlayer/satlayer-bvs/cosmwasm-schema/directory"

	"github.com/satlayer/satlayer-bvs/babylond/cw20"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/examples/squaring/aggregator/svc"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/satlayer/satlayer-bvs/examples/squaring/aggregator/core"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/modules/redis"
)

type ContractsApi struct {
	DelegationManagerApi *api.DelegationManager
	SlashManagerApi      *api.SlashManager
	StrategyBaseApi      *api.StrategyBase
	StrategyManagerApi   *api.StrategyManager
	DirectoryApi         *api.Directory
}

type TestSuite struct {
	suite.Suite
	Ctx      context.Context
	ChainIO  io.ChainIO
	Babylond *babylond.BabylonContainer
	Redis    *RedisContainer
	ContractsApi
	cw20Token        *babylond.DeployedWasmContract
	squaringContract *bvs.Contract[squaringcontract.InstantiateMsg]
}

// SetupSuite Called by tests consumer.
//
// keyDir - points to the `.babylond` folder
// keyName - the keyring info name to use
//
// example:
//
//	 func (suite *ConsumerTestSuite) SetupSuite() {
//		 suite.TestSuite.SetupSuite("keyname")
//	 }
func (suite *TestSuite) SetupSuite(keyDir string, keyName string, privKey string) {
	t := suite.T()
	ctx := context.Background()

	suite.Ctx = ctx
	container := babylond.Run(ctx)
	suite.ChainIO = container.NewChainIO(keyDir)

	suite.Babylond = container

	suite.Babylond.ImportPrivKey("owner", privKey)

	// fund wallets
	suite.Babylond.FundAddressUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)
	suite.Babylond.FundAddressUbbn("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x", 1e8)
	suite.Babylond.FundAddressUbbn("bbn1nrueqkp0wmujyxuqp952j8mnxngm5gek3fsgrj", 1e8) // operator2
	suite.Babylond.FundAddressUbbn("bbn1huw8yau3aqdsp9lr2f85v5plfd46tu026wylaj", 1e8) // wallet1

	redisContainer := suite.StartRedis()
	suite.Redis = redisContainer
	redisUrl := fmt.Sprintf("%s:%s", suite.Redis.Host, suite.Redis.Port)

	// setup keyring
	chainIO, err := suite.ChainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	suite.ChainIO = chainIO

	// deploy BVS contracts
	suite.DeployBvsContracts()

	// deploy squaring contract
	squaringContract := suite.DeploySquaringContract()
	suite.squaringContract = squaringContract

	// register squaring contract into directory
	// squaringContract.Address
	res, err := suite.DirectoryApi.ServiceRegister(ctx, directory.ServiceMetadata{})
	assert.NoError(t, err)
	assert.NotNil(t, res)

	// init env
	core.InitConfig(core.Config{
		Database: core.Database{
			RedisHost: redisUrl,
		},
		Owner: core.Owner{
			KeyName:        keyName,
			KeyDir:         keyDir,
			Bech32Prefix:   "bbn",
			KeyringBackend: "test",
		},
		Chain: core.Chain{
			ID:           suite.Babylond.ChainId,
			RPC:          suite.Babylond.RpcUri,
			BvsDirectory: suite.DirectoryApi.ContractAddr,
			BvsContract:  squaringContract.Address,
		},
	})
	svc.InitMonitor()
}

func (suite *TestSuite) TearDownSuite() {
	suite.Require().NoError(suite.Babylond.Terminate(suite.Ctx))
	suite.Require().NoError(suite.Redis.Container.Terminate(suite.Ctx))
}

type RedisContainer struct {
	Container testcontainers.Container
	Host      string
	Port      string
}

func (suite *TestSuite) StartRedis() *RedisContainer {
	t := suite.T()

	rc, err := redis.Run(suite.Ctx,
		"redis:7",
	)
	assert.NoError(t, err)

	host, err := rc.Host(suite.Ctx)
	assert.NoError(t, err)

	port, err := rc.MappedPort(suite.Ctx, "6379")
	assert.NoError(t, err)

	return &RedisContainer{
		Container: rc,
		Host:      host,
		Port:      port.Port(),
	}
}

func (suite *TestSuite) DeployBvsContracts() {
	tempAddress := suite.Babylond.GenerateAddress("temp:temp")
	deployer := &bvs.Deployer{
		BabylonContainer: suite.Babylond,
	}

	// use temp address for delegation manager + pauser

	pauser := deployer.DeployPauser(nil)

	directoryContract := deployer.DeployDirectory(pauser.Address)
	suite.DirectoryApi = api.NewDirectory(suite.ChainIO, directoryContract.Address)

	strategyManagerContract := deployer.DeployStrategyManager(pauser.Address)
	suite.StrategyManagerApi = api.NewStrategyManager(suite.ChainIO)
	suite.StrategyManagerApi.BindClient(strategyManagerContract.Address)

	delegationManagerContract := deployer.DeployDelegationManager(pauser.Address, 100, []string{tempAddress.String()}, []int64{50})
	suite.DelegationManagerApi = api.NewDelegationManager(suite.ChainIO, delegationManagerContract.Address)

	slashManagerContract := deployer.DeploySlashManager(pauser.Address)
	suite.SlashManagerApi = api.NewSlashManager(suite.ChainIO)
	suite.SlashManagerApi.BindClient(slashManagerContract.Address)

	// deploy CW20 contract
	token := cw20.DeployCw20(deployer.BabylonContainer, cw20.InstantiateMsg{
		Decimals: 18,
		InitialBalances: []cw20.Cw20Coin{
			{
				Address: tempAddress.String(),
				Amount:  "1000000000",
			},
		},
		Mint: &cw20.MinterResponse{
			Minter: tempAddress.String(),
		},
		Name:   "Test Token",
		Symbol: "TEST",
	})
	suite.cw20Token = token

	strategyBaseContract := deployer.DeployStrategyBase(pauser.Address, token.Address, strategyManagerContract.Address)
	suite.StrategyBaseApi = api.NewStrategyBase(suite.ChainIO)
	suite.StrategyBaseApi.BindClient(strategyBaseContract.Address)

	// connect contracts together

	res, err := suite.DelegationManagerApi.SetRouting(context.Background(), strategyManagerContract.Address, slashManagerContract.Address)
	suite.NoError(err)
	suite.Equal(uint32(0), res.TxResult.Code)

	res, err = suite.DirectoryApi.SetRouting(suite.Ctx, delegationManagerContract.Address)
	suite.NoError(err)
	suite.Equal(uint32(0), res.TxResult.Code)

	res, err = suite.SlashManagerApi.SetRouting(context.Background(), delegationManagerContract.Address, strategyManagerContract.Address)
	suite.NoError(err)
	suite.Equal(uint32(0), res.TxResult.Code)

	res, err = suite.StrategyManagerApi.SetRouting(context.Background(), delegationManagerContract.Address, slashManagerContract.Address)
	suite.NoError(err)
	suite.Equal(uint32(0), res.TxResult.Code)

}

// DeploySquaringContract deploys the squaring contract
func (suite *TestSuite) DeploySquaringContract() *bvs.Contract[squaringcontract.InstantiateMsg] {
	t := suite.T()

	aggregatorAccount, err := suite.ChainIO.GetCurrentAccount()
	assert.NoError(t, err)
	aggregatorAddress := aggregatorAccount.GetAddress()

	// load squaring contract wasm bytecode
	_, currentFile, _, _ := runtime.Caller(0)
	baseDir := filepath.Dir(currentFile)
	targetFile := filepath.Join(baseDir, "../../node_modules/@examples/squaring-contract/dist/contract.wasm")
	wasmByteCode, err := os.ReadFile(targetFile)
	assert.NoError(t, err)

	initMsg := squaringcontract.InstantiateMsg{
		Aggregator: aggregatorAddress.String(),
	}
	initBytes, err := json.Marshal(initMsg)
	assert.NoError(t, err)

	// deploy and init squaring contract
	contract, err := suite.Babylond.StoreAndInitWasm(wasmByteCode, initBytes, "squaring contract", "genesis")
	assert.NoError(t, err)

	return &bvs.Contract[squaringcontract.InstantiateMsg]{
		DeployedWasmContract: *contract,
		InstantiateMsg:       initMsg,
	}
}

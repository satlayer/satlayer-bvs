package tests

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-cw/driver"
	statebank "github.com/satlayer/satlayer-bvs/bvs-cw/state-bank"
	"github.com/satlayer/satlayer-bvs/examples/squaring/aggregator/core"
	"github.com/satlayer/satlayer-bvs/examples/squaring/aggregator/svc"
	squaringcontract "github.com/satlayer/satlayer-bvs/examples/squaring/squaring-contract"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/modules/redis"
)

type TestSuite struct {
	suite.Suite
	Ctx      context.Context
	ChainIO  io.ChainIO
	Babylond *babylond.BabylonContainer
	Redis    *RedisContainer
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
func (suite *TestSuite) SetupSuite(keyDir string, keyName string) {
	t := suite.T()
	ctx := context.Background()
	suite.Ctx = ctx
	container := babylond.Run(ctx)
	suite.ChainIO = container.NewChainIO(keyDir)
	suite.Babylond = container

	redisContainer := suite.StartRedis()
	suite.Redis = redisContainer
	redisUrl := fmt.Sprintf("%s:%s", suite.Redis.Host, suite.Redis.Port)

	// init env
	core.InitConfig(core.Config{
		Database: core.Database{
			RedisHost: redisUrl,
		},
		Owner: core.Owner{
			KeyName: keyName,
			KeyDir:  keyDir,
		},
	})
	svc.InitMonitor()

	// setup keyring
	chainIO, err := suite.ChainIO.SetupKeyring(core.C.Owner.KeyName, core.C.Owner.KeyringBackend)
	assert.NoError(t, err)
	suite.ChainIO = chainIO
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

// deployBVSContracts deploys the dependency contracts for the squaring contract (statebank and driver)
func (suite *TestSuite) deployBVSContracts() (*bvs.Contract[statebank.InstantiateMsg], *bvs.Contract[driver.InstantiateMsg]) {

	deployer := bvs.Deployer{
		BabylonContainer: suite.Babylond,
	}

	stateBankContract := deployer.DeployStateBank()
	driverContract := deployer.DeployDriver()

	return stateBankContract, driverContract
}

// DeploySquaringContract deploys the squaring contract
func (suite *TestSuite) DeploySquaringContract() *bvs.Contract[squaringcontract.InstantiateMsg] {
	stateBankContract, driverContract := suite.deployBVSContracts()
	t := suite.T()

	aggregatorAccount, err := suite.ChainIO.GetCurrentAccount()
	assert.NoError(t, err)
	aggregatorAddress := aggregatorAccount.GetAddress()

	// load squaring contract wasm bytecode
	_, currentFile, _, _ := runtime.Caller(0)
	baseDir := filepath.Dir(currentFile)
	targetFile := filepath.Join(baseDir, "../../node_modules/@examples/squaring-contract/artifacts/squaring_contract.wasm")
	wasmByteCode, err := os.ReadFile(targetFile)
	assert.NoError(t, err)

	initMsg := squaringcontract.InstantiateMsg{
		Aggregator: aggregatorAddress.String(),
		BvsDriver:  driverContract.Address,
		StateBank:  stateBankContract.Address,
	}
	initBytes, err := json.Marshal(initMsg)
	assert.NoError(t, err)

	// deploy and init squaring contract
	contract, err := suite.Babylond.StoreAndInitWasm(wasmByteCode, initBytes, "squaring contract", "genesis")
	assert.NoError(t, err)

	// register with statebank
	stateBank := api.NewStateBank(suite.ChainIO)
	stateBank.BindClient(stateBankContract.Address)
	_, err = stateBank.SetRegisteredBVSContract(suite.Ctx, contract.Address)
	assert.NoError(t, err)

	// register with bvsdriver
	bvsDriver := api.NewDriver(suite.ChainIO)
	bvsDriver.BindClient(driverContract.Address)
	_, err = bvsDriver.SetRegisteredBVSContract(suite.Ctx, contract.Address)
	assert.NoError(t, err)

	return &bvs.Contract[squaringcontract.InstantiateMsg]{
		DeployedWasmContract: *contract,
		InstantiateMsg:       initMsg,
	}
}

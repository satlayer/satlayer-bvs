package bvsdriverevm

import (
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	logger2 "github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
	abis "github.com/satlayer/satlayer-bvs/bvs-cli/conf/abi"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

type Service struct {
	ChainIO   io.ETHChainIO
	BVSDriver api.ETHBVSDriver
}

func NewService() *Service {
	conf.InitConfig()
	logger := logger2.NewELKLogger("satlayer-cli")
	logger.SetLogLevel(conf.C.LogLevel)
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "bvs_driver_evm")
	ethChainIO, err := io.NewETHChainIO(conf.C.Chain.EVMRPC, conf.C.Account.EVMKeyDir, logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:                 3,
		RetryInterval:              2 * time.Second,
		ConfirmationTimeout:        60 * time.Second,
		ETHGasFeeCapAdjustmentRate: 2,
		ETHGasLimitAdjustmentRate:  1.1,
		GasLimit:                   1000000000,
	})
	if err != nil {
		panic(err)
	}

	if conf.C.Contract.BVSDriverEVM == "" {
		panic("Contract address for BVSDriverEVM is empty!")
	}

	contractABI, err := abi.JSON(strings.NewReader(abis.BVSDriverABI))
	if err != nil {
		panic(err)
	}
	bvsDriverContract := common.HexToAddress(conf.C.Contract.BVSDriverEVM)

	bvsDriver := api.NewETHBVSDriverImpl(ethChainIO, bvsDriverContract, &contractABI)
	return &Service{ChainIO: ethChainIO, BVSDriver: bvsDriver}
}

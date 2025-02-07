package statebankevm

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
	abis "github.com/satlayer/satlayer-bvs/satlayer-cli/conf/abi"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/conf"
)

type Service struct {
	ChainIO   io.ETHChainIO
	StateBank api.ETHStateBank
}

func NewService() *Service {
	conf.InitConfig()
	logger := logger2.NewELKLogger("satlayer-cli")
	logger.SetLogLevel(conf.C.LogLevel)
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "state_bank_evm")
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

	if conf.C.Contract.StateBankEVM == "" {
		panic("Contract address for StateBankEVM is empty!")
	}

	contractABI, err := abi.JSON(strings.NewReader(abis.StateBankABI))
	if err != nil {
		panic(err)
	}
	stateBankContract := common.HexToAddress(conf.C.Contract.StateBankEVM)

	stateBank := api.NewETHStateBankImpl(ethChainIO, stateBankContract, &contractABI)
	return &Service{ChainIO: ethChainIO, StateBank: stateBank}
}

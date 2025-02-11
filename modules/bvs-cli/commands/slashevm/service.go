package slashevm

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
	ChainIO io.ETHChainIO
	Slash   api.ETHSlashManager
}

func NewService() *Service {
	conf.InitConfig()
	logger := logger2.NewELKLogger("satlayer-cli")
	logger.SetLogLevel(conf.C.LogLevel)
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "slash_evm")
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

	if conf.C.Contract.SlashEVM == "" {
		panic("Contract address for Slash is empty!")
	}

	contractABI, err := abi.JSON(strings.NewReader(abis.SlashManagerABI))
	if err != nil {
		panic(err)
	}
	slashContract := common.HexToAddress(conf.C.Contract.SlashEVM)
	slashManager := api.NewETHSlashManagerImpl(ethChainIO, slashContract, &contractABI)
	return &Service{ChainIO: ethChainIO, Slash: slashManager}
}

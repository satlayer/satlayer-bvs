package main

import (
	"context"
	"fmt"
	"time"

	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"

	"github.com/satlayer/satlayer-bvs/examples/squaring/initprogram/core"
)

func main() {
	core.InitConfig()
	approverAddress := getApproverAccount()
	print("approverAddress: ", approverAddress)
	registerBvsContract()
	registerOperators(approverAddress)
	registerStrategy()
	registerStakers("0")
}

func getApproverAccount() string {
	elkLogger := logger.NewELKLogger("bvs_demo")
	reg := prometheus.NewRegistry()
	metricsIndicators := transactionprocess.NewPromIndicators(reg, "bvs_demo")
	approverClient, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Account.KeyDir, core.C.Account.Bech32Prefix, elkLogger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             5,
		RetryInterval:          3 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})

	if err != nil {
		panic(err)
	}

	approverClient, err = approverClient.SetupKeyring(core.C.Account.ApproverKeyName, core.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}
	pubKey := approverClient.GetCurrentAccountPubKey()
	approverAddress := sdktypes.AccAddress(pubKey.Address()).String()

	return approverAddress
}

func registerBvsContract() string {
	elkLogger := logger.NewELKLogger("bvs_demo")
	elkLogger.SetLogLevel("info")
	reg := prometheus.NewRegistry()
	metricsIndicators := transactionprocess.NewPromIndicators(reg, "bvs_demo")
	chainIO, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Account.KeyDir, core.C.Account.Bech32Prefix, elkLogger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             5,
		RetryInterval:          3 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}

	chainIO, err = chainIO.SetupKeyring(core.C.Account.CallerKeyName, core.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}

	txResp, err := api.NewDirectory(chainIO, core.C.Contract.DirectoryAddr).RegisterBvs(context.Background(), core.C.Contract.BVSContractAddr)
	if err != nil {
		panic(err)
	}
	fmt.Printf("registerBvsContract success, txn: %s\n", txResp.Hash.String())
	return txResp.Hash.String()
}

func registerOperators(approverAddress string) {
	elkLogger := logger.NewELKLogger("bvs_demo")
	elkLogger.SetLogLevel("info")
	reg := prometheus.NewRegistry()
	metricsIndicators := transactionprocess.NewPromIndicators(reg, "bvs_demo")
	chainIO, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Account.KeyDir, core.C.Account.Bech32Prefix, elkLogger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             5,
		RetryInterval:          3 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}

	for _, operator := range core.C.Account.OperatorsKeyName {
		chainIO, err = chainIO.SetupKeyring(operator, core.C.Account.KeyringBackend)
		if err != nil {
			panic(err)
		}

		pubKey := chainIO.GetCurrentAccountPubKey()
		address := sdktypes.AccAddress(pubKey.Address()).String()
		delegation := api.NewDelegationManager(chainIO, core.C.Contract.DelegationManagerAddr)
		txResp, err := delegation.RegisterAsOperator(
			context.Background(),
			pubKey,
			"",
			approverAddress,
			"",
			0,
		)
		if err != nil {
			fmt.Println("Ere registerAsOperator to delegation failed: ", err)
		} else {
			fmt.Println("registerAsOperator to delegation success:", txResp)
		}
		// register operator to bvsDirectory
		txResp, err = api.NewDirectory(chainIO, core.C.Contract.DirectoryAddr).RegisterOperator(context.Background(), address, pubKey)
		if err != nil {
			fmt.Println("Err: registerOperators to bvsDirectory failed: ", err)
			return
		}
		fmt.Println("registerOperators to bvsDirectory success:", txResp)
		return

	}
}

func registerStrategy() {
	fmt.Println("registerStrategy")
	elkLogger := logger.NewELKLogger("bvs_demo")
	elkLogger.SetLogLevel("info")
	reg := prometheus.NewRegistry()
	metricsIndicators := transactionprocess.NewPromIndicators(reg, "bvs_demo")
	chainIO, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Account.KeyDir, core.C.Account.Bech32Prefix, elkLogger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             5,
		RetryInterval:          3 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}
	fmt.Println("set register")
	chainIO, err = chainIO.SetupKeyring(core.C.Account.StrategyManagerKeyName, core.C.Account.KeyringBackend)
	strategyManager := api.NewStrategyManager(chainIO, core.C.Contract.StrategyMangerAddr)
	ctx := context.Background()
	fmt.Println("hello.....")

	// register delegation manager
	resp, err := strategyManager.SetDelegationManager(ctx, core.C.Contract.DelegationManagerAddr)
	if err != nil {
		fmt.Println("Err: setDelegationManager failed: ", err)
	} else {
		fmt.Println("SetDelegationManager success:", resp)
	}

	resp, err = strategyManager.AddStrategiesToWhitelist(ctx, []string{core.C.Contract.StrategyAddr}, []bool{false})
	if err != nil {
		fmt.Println("Err: addStrategiesToWhitelist failed: ", err)
	} else {
		fmt.Println("AddStrategiesToWhitelist success:", resp)
	}
}

func registerStakers(approverAddress string) {
	elkLogger := logger.NewELKLogger("bvs_demo")
	elkLogger.SetLogLevel("info")

	reg := prometheus.NewRegistry()
	metricsIndicators := transactionprocess.NewPromIndicators(reg, "bvs_demo")
	chainIO, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Account.KeyDir, core.C.Account.Bech32Prefix, elkLogger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             5,
		RetryInterval:          3 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}

	for _, staker := range core.C.StakerOperatorMap {
		fmt.Printf("staker: %+v\n", staker)

		sclient, err := chainIO.SetupKeyring(staker.StakerKeyName, core.C.Account.KeyringBackend)
		if err != nil {
			panic(err)
		}

		delegation := api.NewDelegationManager(sclient, core.C.Contract.DelegationManagerAddr)
		oClient, err := chainIO.SetupKeyring(staker.OperatorKeyName, core.C.Account.KeyringBackend)
		if err != nil {
			panic(err)
		}

		nodeStatus, err := chainIO.QueryNodeStatus(context.Background())
		if err != nil {
			panic(err)
		}
		expiry := nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000

		pubKey := oClient.GetCurrentAccountPubKey()
		address := sdktypes.AccAddress(pubKey.Address()).String()
		txResp, err := delegation.DelegateTo(
			context.Background(),
			address,
			approverAddress,
			core.C.Account.ApproverKeyName,
			pubKey,
			expiry,
		)
		if err != nil {
			fmt.Println("Err: ", err)
		}
		fmt.Println("DelegateTo to operator success:", txResp)

		txnResp, err := api.IncreaseTokenAllowance(context.Background(), sclient, 9999999999999999, core.C.Contract.Cw20TokenAddr, core.C.Contract.StrategyMangerAddr, sdktypes.NewInt64DecCoin("ubbn", 1))
		if err != nil {
			fmt.Println("Err: ", err)
		}
		fmt.Println("increaseTokenAllowance success:", txnResp)

		// register staker to strategy
		strategyManager := api.NewStrategyManager(sclient, core.C.Contract.StrategyMangerAddr)
		resp, err := strategyManager.DepositIntoStrategy(context.Background(), core.C.Contract.StrategyAddr, core.C.Contract.Cw20TokenAddr, staker.Amount)
		if err != nil {
			err := fmt.Errorf("DepositIntoStrategy failed: %v", err)
			fmt.Println("Err", err)
		} else {
			fmt.Println("DepositIntoStrategy success:", resp)
		}

	}
}

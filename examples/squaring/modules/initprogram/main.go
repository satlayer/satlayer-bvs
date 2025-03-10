package main

import (
	"context"
	"fmt"
	"time"

	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/examples/squaring/initprogram/core"
)

func main() {
	core.InitConfig()
	approverAddress := getApproverAccount()
	print("approverAddress: ", approverAddress)
	//registerBvsContract()
	//registerOperators()
	registerStrategy()
	registerStakers()
}

func getApproverAccount() string {
	approverClient, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Account.KeyDir, core.C.Account.Bech32Prefix, types.TxManagerParams{
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

//func registerBvsContract() string {
//	chainIO, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Account.KeyDir, core.C.Account.Bech32Prefix, types.TxManagerParams{
//		MaxRetries:             5,
//		RetryInterval:          3 * time.Second,
//		ConfirmationTimeout:    60 * time.Second,
//		GasPriceAdjustmentRate: "1.1",
//	})
//	if err != nil {
//		panic(err)
//	}
//
//	chainIO, err = chainIO.SetupKeyring(core.C.Account.CallerKeyName, core.C.Account.KeyringBackend)
//	if err != nil {
//		panic(err)
//	}
//
//	txResp, err := api.NewDirectory(chainIO, core.C.Contract.DirectoryAddr).RegisterBvs(context.Background(), core.C.Contract.BVSContractAddr)
//	if err != nil {
//		panic(err)
//	}
//	fmt.Printf("registerBvsContract success, txn: %s\n", txResp.Hash.String())
//	return txResp.Hash.String()
//}
//
//func registerOperators() {
//	chainIO, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Account.KeyDir, core.C.Account.Bech32Prefix, types.TxManagerParams{
//		MaxRetries:             5,
//		RetryInterval:          3 * time.Second,
//		ConfirmationTimeout:    60 * time.Second,
//		GasPriceAdjustmentRate: "1.1",
//	})
//	if err != nil {
//		panic(err)
//	}
//
//	for _, operator := range core.C.Account.OperatorsKeyName {
//		chainIO, err = chainIO.SetupKeyring(operator, core.C.Account.KeyringBackend)
//		if err != nil {
//			panic(err)
//		}
//
//		pubKey := chainIO.GetCurrentAccountPubKey()
//		address := sdktypes.AccAddress(pubKey.Address()).String()
//		delegation := api.NewDelegationManager(chainIO, core.C.Contract.DelegationManagerAddr)
//		txResp, err := delegation.RegisterAsOperator(
//			context.Background(),
//			"",
//			0,
//		)
//		if err != nil {
//			fmt.Println("Ere registerAsOperator to delegation failed: ", err)
//		} else {
//			fmt.Println("registerAsOperator to delegation success:", txResp)
//		}
//		// register operator to bvsDirectory
//		txResp, err = api.NewDirectory(chainIO, core.C.Contract.DirectoryAddr).RegisterOperator(context.Background(), address, pubKey)
//		if err != nil {
//			fmt.Println("Err: registerOperators to bvsDirectory failed: ", err)
//			return
//		}
//		fmt.Println("registerOperators to bvsDirectory success:", txResp)
//		return
//
//	}
//}

func registerStrategy() {
	fmt.Println("registerStrategy")
	chainIO, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Account.KeyDir, core.C.Account.Bech32Prefix, types.TxManagerParams{
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
	strategyManager := api.NewStrategyManager(chainIO)
	strategyManager.BindClient(core.C.Contract.StrategyMangerAddr)
	ctx := context.Background()
	fmt.Println("hello.....")

	resp, err := strategyManager.UpdateStrategy(ctx, core.C.Contract.StrategyAddr, true)
	if err != nil {
		fmt.Println("Err: addStrategiesToWhitelist failed: ", err)
	} else {
		fmt.Println("UpdateStrategy success:", resp)
	}
}

func registerStakers() {
	chainIO, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Account.KeyDir, core.C.Account.Bech32Prefix, types.TxManagerParams{
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

		pubKey := oClient.GetCurrentAccountPubKey()
		operator := sdktypes.AccAddress(pubKey.Address()).String()

		txResp, err := delegation.DelegateTo(
			context.Background(),
			operator,
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
		strategyManager := api.NewStrategyManager(sclient)
		strategyManager.BindClient(core.C.Contract.StrategyMangerAddr)
		resp, err := strategyManager.DepositIntoStrategy(context.Background(), core.C.Contract.StrategyAddr, core.C.Contract.Cw20TokenAddr, staker.Amount)
		if err != nil {
			err := fmt.Errorf("DepositIntoStrategy failed: %v", err)
			fmt.Println("Err", err)
		} else {
			fmt.Println("DepositIntoStrategy success:", resp)
		}

	}
}

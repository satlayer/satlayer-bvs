package delegation

import (
	"encoding/base64"
	"fmt"

	delegationmanager "github.com/satlayer/satlayer-bvs/bvs-cw/delegation-manager"
)

func IsDelegated(stakerAddress string) {
	s := NewService()
	resp, err := s.Delegation.IsDelegated(stakerAddress)
	if err != nil {
		panic(err)
	}
	isDelegated := resp.IsDelegated
	fmt.Printf("%t\n", isDelegated)
}

func IsOperator(operatorAddress string) {
	s := NewService()
	resp, err := s.Delegation.IsOperator(operatorAddress)
	if err != nil {
		panic(err)
	}
	isOperator := resp.IsOperator
	fmt.Printf("%t\n", isOperator)
}

func GetOperatorDetails(operatorAddress string) {
	s := NewService()
	resp, err := s.Delegation.OperatorDetails(operatorAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("DelegationApprover: %s\nDeprecatedEarningsReceiver: %s\nStakerOptOutWindowBlocks: %d",
		resp.Details.DelegationApprover, resp.Details.DeprecatedEarningsReceiver, resp.Details.StakerOptOutWindowBlocks)
}

func GetDelegationApprover(operatorAddress string) {
	s := NewService()
	resp, err := s.Delegation.DelegationApprover(operatorAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.DelegationApprover)
}

func GetStakerOptOutWindowBlocks(operatorAddress string) {
	s := NewService()
	resp, err := s.Delegation.StakerOptOutWindowBlocks(operatorAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%d\n", resp.StakerOptOutWindowBlocks)
}

func GetOperatorShares(operatorAddress string, strategies []string) {
	s := NewService()
	resp, err := s.Delegation.GetOperatorShares(operatorAddress, strategies)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Shares)
}

func GetOperatorStakers(operatorAddress string) {
	s := NewService()
	resp, err := s.Delegation.GetOperatorStakers(operatorAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.StakersAndShares)
}

func GetDelegatableShares(operatorAddress string) {
	s := NewService()
	resp, err := s.Delegation.GetDelegatableShares(operatorAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Shares)
}

func GetWithdrawDelay(strategyAddress []string) {
	s := NewService()
	resp, err := s.Delegation.GetWithdrawalDelay(strategyAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%d\n", resp.WithdrawalDelays)
}

func GetStakerNonce(stakerAddress string) {
	s := NewService()
	resp, err := s.Delegation.GetStakerNonce(stakerAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Nonce)
}

func CalcWithdrawRoot(stakerAddress, delegatedAddress, withdrawerAddress, nonce string, startBlock int64, strategies, shares []string) {
	s := NewService()
	resp, err := s.Delegation.CalculateWithdrawalRoot(delegationmanager.CalculateWithdrawalRootWithdrawal{
		Staker:      stakerAddress,
		DelegatedTo: delegatedAddress,
		Withdrawer:  withdrawerAddress,
		Nonce:       nonce,
		StartBlock:  startBlock,
		Strategies:  strategies,
		Shares:      shares,
	})
	if err != nil {
		panic(err)
	}
	root := base64.StdEncoding.EncodeToString(resp)
	fmt.Printf("%s\n", root)
}

func GetCumulativeWithdrawQueueNonce(stakerAddress string) {
	s := NewService()
	resp, err := s.Delegation.GetCumulativeWithdrawalsQueuedNonce(stakerAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.CumulativeWithdrawals)
}

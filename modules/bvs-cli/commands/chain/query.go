package chain

import (
	"context"
	"fmt"
)

func QueryNode() {
	s := NewService()
	ctx := context.Background()
	resp, err := s.ChainIO.QueryNodeStatus(ctx)
	if err != nil {
		fmt.Printf("Failed. Error msg: %+v", err)
		return
	}
	fmt.Printf(
		"===NodeInfo===\n1. NodeVersion: %s\n2. ProtocolVersion: P2P: %d, Block: %d, App: %d\n3. DefaultNodeID: %s\n4. ListenAddr: %s\n5. Network: %s\n"+
			"===SyncInfo===\n1. CatchingUp: %t\n2. LatestBlockTime: %s\n3. LatestBlockHash: %s\n4. LatestBlockHeight: %d\n"+
			"===ValidatorInfo===\n1. Address: %s\n2. VotingPower: %d\n",
		resp.NodeInfo.Version, resp.NodeInfo.ProtocolVersion.P2P, resp.NodeInfo.ProtocolVersion.Block, resp.NodeInfo.ProtocolVersion.App, resp.NodeInfo.DefaultNodeID,
		resp.NodeInfo.ListenAddr, resp.NodeInfo.Network,
		resp.SyncInfo.CatchingUp, resp.SyncInfo.LatestBlockTime, resp.SyncInfo.LatestBlockHash, resp.SyncInfo.LatestBlockHeight,
		resp.ValidatorInfo.Address, resp.ValidatorInfo.VotingPower,
	)
}

func QueryTxn(txnHash string) {
	s := NewService()
	resp, err := s.ChainIO.QueryTransaction(txnHash)

	if err != nil {
		fmt.Printf("Query Txn Failed. Error msg: %s", err)
		return
	}
	fmt.Printf("===TxnInfo===\n1. Index: %d\n2. TxHash: %s\n3. Height: %d\n4. Events: %+v\n", resp.Index, resp.Hash.String(), resp.Height, resp.TxResult.Events)
}

func QueryAccount(account string) {
	s := NewService()
	resp, err := s.ChainIO.QueryAccount(account)
	if err != nil {
		fmt.Printf("Failed. Error msg: %+v", err)
		return
	}
	fmt.Printf("===AccountInfo===\n1. Address: %s\n2. AccountNumber: %d\n3. Sequence: %d\n", resp.GetAddress(), resp.GetAccountNumber(), resp.GetSequence())
}

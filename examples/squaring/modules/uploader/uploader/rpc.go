package uploader

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	rewardscoordinator "github.com/satlayer/satlayer-bvs/bvs-cw/rewards-coordinator"
	"math"
	"strconv"
	"time"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
)

const calcInterval = 86_400 // 1 day

type MerkleNode struct {
	Left  *MerkleNode
	Right *MerkleNode
	Hash  string
}

type HashResponse struct {
	HashBinary []byte `json:"hash_binary"`
}

type MerkleizeLeavesResponse struct {
	RootHashBinary []byte `json:"root_hash_binary"`
}

type EarnerLeafHashResponse struct {
	RootHashBinary []byte `json:"hash_binary"`
}

type UnderlyingTokenResponse struct {
	UnderlyingTokenAddr string `json:"underlying_token_addr"`
}

// rpcSubmission sends a CreateRewardsForAllSubmission transaction to the rewards coordinator.
//
// Given a list of Submission, this method creates a RewardsSubmission for each submission,
// and sends a CreateRewardsForAllSubmission transaction to the rewards coordinator with the
// list of submissions. The start time of the submission is always the current time minus the
// remainder of dividing the current time by calcInterval, rounded down. The duration of the
// submission is always calcInterval.
//
// The method returns an error if the transaction fails.
func (u *Uploader) rpcSubmission(rewards []Submission) error {
	now := time.Now().Unix()
	startTime := now - now%calcInterval
	submissions := make([]rewardscoordinator.RewardsSubmission, 0)

	for _, reward := range rewards {
		submissions = append(submissions, rewardscoordinator.RewardsSubmission{
			StrategiesAndMultipliers: []rewardscoordinator.StrategyAndMultiplier{
				{
					Strategy:   reward.Strategy,
					Multiplier: 1,
				},
			},
			Token:          reward.Token,
			Amount:         strconv.FormatFloat(math.Floor(reward.Amount), 'f', -1, 64),
			StartTimestamp: fmt.Sprintf("%d000000000", startTime),
			Duration:       calcInterval,
		})
	}
	fmt.Printf("submissions: %+v\n", submissions)
	resp, err := u.rewardsCoordinator.CreateRewardsForAllSubmission(context.Background(), submissions)
	if err != nil {
		fmt.Println("CreateRewardsForAllSubmission err: ", err)
		return err
	}
	fmt.Println("CreateRewardsForAllSubmission txn hash: ", resp.Hash.String())
	return err
}

func (u *Uploader) rpcTokenHash(token *TokenAmount) (string, error) {
	resp, err := u.rewardsCoordinator.CalculateTokenLeafHash(token.Token, token.RewardAmount)
	if err != nil {
		fmt.Println("CalculateTokenLeafHash err: ", err)
		return "", err
	}
	var hashResponse HashResponse
	if err := json.Unmarshal(resp.Data, &hashResponse); err != nil {
		fmt.Println("unmarshal err: ", err)
		return "", err
	}
	hashStr := base64.StdEncoding.EncodeToString(hashResponse.HashBinary)
	return hashStr, err
}

func (u *Uploader) rpcMerkleizeLeaves(leaves []string) (string, error) {
	resp, err := u.rewardsCoordinator.MerkleizeLeaves(leaves)
	if err != nil {
		fmt.Println("merkleizeLeaves err: ", err)
		return "", err
	}
	var merkleizeLeavesResponse MerkleizeLeavesResponse
	if err := json.Unmarshal(resp.Data, &merkleizeLeavesResponse); err != nil {
		fmt.Println("unmarshal err: ", err)
		return "", err
	}
	merkleRoot := base64.StdEncoding.EncodeToString(merkleizeLeavesResponse.RootHashBinary)
	return merkleRoot, err
}

func (u *Uploader) rpcEarnerLeafHash(staker, rootHash string) (string, error) {
	resp, err := u.rewardsCoordinator.CalculateEarnerLeafHash(staker, rootHash)
	if err != nil {
		fmt.Println("CalculateEarnerLeafHash err: ", err)
		return "", err
	}
	var earnerLeafHashResponse EarnerLeafHashResponse
	err = json.Unmarshal(resp.Data, &earnerLeafHashResponse)
	hashStr := base64.StdEncoding.EncodeToString(earnerLeafHashResponse.RootHashBinary)
	return hashStr, err
}

// rpcUnderlyingToken queries the underlying token address for the given strategy
// and returns it as a string. If the query fails, an error is returned.
func (u *Uploader) rpcUnderlyingToken(strategy string) (string, error) {
	strategyBase := api.NewStrategyBase(u.chainIO)
	strategyBase.BindClient(strategy)
	resp, err := strategyBase.UnderlyingToken()
	if err != nil {
		return "", err
	}
	var tokenRsp UnderlyingTokenResponse
	err = json.Unmarshal(resp.Data, &tokenRsp)

	return tokenRsp.UnderlyingTokenAddr, nil
}

// rpcSubmitHashRoot submits a root hash to the rewards coordinator. The root hash is submitted with a
// timestamp that is one hour ago from the current time.
//
// The method returns an error if the transaction fails.
func (u *Uploader) rpcSubmitHashRoot(rootHash string) error {
	timestamp := time.Now().Unix() - 3600
	rsp, err := u.rewardsCoordinator.SubmitRoot(context.Background(), rootHash, timestamp)
	if err != nil {
		fmt.Println("SubmitRootHash err: ", err)
		return err
	}
	fmt.Println("SubmitRootHash txn hash: ", rsp.Hash.String())
	return err
}

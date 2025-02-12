package uploader

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	rio "io"
	"net/http"
	"strconv"
	"time"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/shopspring/decimal"

	"github.com/satlayer/satlayer-bvs/examples/squaring/uploader/core"
)

const calcInterval = 86_400 // 1 day

type MerkleNode struct {
	Hash  string      `json:"hash"`
	Left  *MerkleNode `json:"left,omitempty"`
	Right *MerkleNode `json:"right,omitempty"`
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
	submissions := make([]types.RewardsSubmission, 0)

	for _, reward := range rewards {
		submissions = append(submissions, types.RewardsSubmission{
			StrategiesAndMultipliers: []types.StrategyAndMultiplier{{
				Strategy:   reward.Strategy,
				Multiplier: 1,
			}},
			Token:          reward.Token,
			Amount:         reward.Amount.Ceil().String(),
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
func (u *Uploader) rpcSubmitHashRoot(rootHash string, earners []*RewardEarner, totalEarnerTokenAmount []EarnerTokenAmount) error {
	nowTs := time.Now().Unix()
	timestamp := nowTs - 3600
	rsp, err := u.rewardsCoordinator.SubmitRoot(context.Background(), rootHash, uint64(timestamp))
	if err != nil {
		fmt.Println("SubmitRootHash err: ", err)
		return err
	}
	txnHash := rsp.Hash.String()
	rootIndex := 0
	rewardsCalculationEndTimestamp := 0
	activatedAt := 0
	for _, event := range rsp.TxResult.Events {
		if event.Type == "wasm-DistributionRootSubmitted" {
			fmt.Println("SubmitRootHash event: ", event.Attributes)
			for _, attr := range event.Attributes {
				if attr.Key == "root_index" {
					rootIndex, _ = strconv.Atoi(attr.Value)
				} else if attr.Key == "rewards_calculation_end_timestamp" {
					rewardsCalculationEndTimestamp, _ = strconv.Atoi(attr.Value)
				} else if attr.Key == "activated_at" {
					activatedAt, _ = strconv.Atoi(attr.Value)
				}
			}
		}
	}
	u.saveRewardEarner(rootIndex, rootHash, txnHash, nowTs, int64(rewardsCalculationEndTimestamp), int64(activatedAt), earners)
	u.updateEarnerTokenAmount(totalEarnerTokenAmount)
	fmt.Println("SubmitRootHash txn hash: ", rsp.Hash.String())
	return err
}

func (u *Uploader) saveRewardEarner(rootIndex int, rootHash, txnHash string, createTs, CalcEndTs, activatedTs int64, earners []*RewardEarner) {
	nowTs := time.Now().Unix()
	earner, err := json.Marshal(earners)
	if err != nil {
		return
	}
	earnerStr := string(earner)
	msgPayload := fmt.Sprintf("%d-%d-%s-%s", nowTs, rootIndex, txnHash, earnerStr)
	msgBytes := []byte(msgPayload)
	signature, err := u.chainIO.GetSigner().Sign(msgBytes)

	payload := RewardUploadRequest{
		RootIndex:   int64(rootIndex),
		RootHash:    rootHash,
		CalcEndTs:   CalcEndTs,
		TxnHash:     txnHash,
		CreateTs:    createTs,
		ActivatedTs: activatedTs,
		Timestamp:   nowTs,
		Signature:   signature,
		PubKey:      u.pubKeyStr,
		Earners:     earners,
	}
	fmt.Printf("task result send aggregator payload: %+v\n", payload)
	if err != nil {
		return
	}
	jsonData, err := json.Marshal(payload)
	if err != nil {
		fmt.Printf("Error marshaling JSON: %s", err)
		return
	}

	resp, err := http.Post(fmt.Sprintf("%s/api/v1/rewards", core.C.Reward.UploadAPI), "application/json", bytes.NewBuffer(jsonData))
	if err != nil {
		fmt.Printf("Error sending reward api : %s\n", err)
		return
	}
	if resp.StatusCode != 200 {
		body, _ := rio.ReadAll(resp.Body)
		fmt.Printf("Error sending rewarda api : %s\n", string(body))
		return
	}
	return
}

func (u *Uploader) getEarnerTokenAmount(earner, token string) (decimal.Decimal, error) {
	zero := decimal.NewFromFloat(0.0)
	resp, err := http.Get(fmt.Sprintf("%s/api/v1/token/%s/%s", core.C.Reward.UploadAPI, earner, token))
	if err != nil {
		fmt.Printf("Error sending reward api : %s\n", err)
		return zero, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != 200 {
		body, _ := rio.ReadAll(resp.Body)
		return zero, fmt.Errorf("error response from api: %s", string(body))
	}

	var result struct {
		Amount string `json:"amount"`
	}

	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return zero, fmt.Errorf("error decoding response: %v", err)
	}

	amount, err := decimal.NewFromString(result.Amount)
	if err != nil {
		return zero, fmt.Errorf("error converting amount to decimal: %v", err)
	}

	return amount, nil
}

func (u *Uploader) updateEarnerTokenAmount(totalEarnerTokenAmount []EarnerTokenAmount) {
	nowTs := time.Now().Unix()
	earner, err := json.Marshal(totalEarnerTokenAmount)
	if err != nil {
		return
	}
	earnerStr := string(earner)
	msgPayload := fmt.Sprintf("%d-%s", nowTs, earnerStr)
	msgBytes := []byte(msgPayload)
	signature, err := u.chainIO.GetSigner().Sign(msgBytes)
	payload := EarnerTokenRequest{
		Timestamp:    nowTs,
		Signature:    signature,
		PubKey:       u.pubKeyStr,
		EarnerTokens: totalEarnerTokenAmount,
	}
	if err != nil {
		return
	}
	jsonData, err := json.Marshal(payload)
	if err != nil {
		fmt.Printf("Error marshaling JSON: %s", err)
		return
	}

	resp, err := http.Post(fmt.Sprintf("%s/api/v1/token", core.C.Reward.UploadAPI), "application/json", bytes.NewBuffer(jsonData))
	if err != nil {
		fmt.Printf("Error sending reward api : %s\n", err)
		return
	}
	if resp.StatusCode != 200 {
		body, _ := rio.ReadAll(resp.Body)
		fmt.Printf("Error sending reward api : %s\n", string(body))
		return
	}
	return
}

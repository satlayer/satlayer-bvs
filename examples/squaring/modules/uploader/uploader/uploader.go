package uploader

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"sort"
	"strings"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/indexer"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
	"github.com/shopspring/decimal"

	"github.com/satlayer/satlayer-bvs/examples/squaring/uploader/core"
)

type Uploader struct {
	bvsContract        string
	delegation         api.Delegation
	chainIO            io.ChainIO
	rewardsCoordinator api.RewardsCoordinator
	pubKeyStr          string
}

func NewUploader() *Uploader {
	elkLogger := logger.NewELKLogger("bvs_demo")
	elkLogger.SetLogLevel("info")

	reg := prometheus.NewRegistry()
	metricsIndicators := transactionprocess.NewPromIndicators(reg, "bvs_demo")
	chainIO, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Owner.KeyDir, core.C.Owner.Bech32Prefix, elkLogger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             5,
		RetryInterval:          3 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}

	client, err := chainIO.SetupKeyring(core.C.Owner.KeyName, core.C.Owner.KeyringBackend)
	if err != nil {
		panic(err)
	}
	pubKey := client.GetCurrentAccountPubKey()
	pubKeyStr := base64.StdEncoding.EncodeToString(pubKey.Bytes())

	txResp, err := api.NewBVSDirectoryImpl(client, core.C.Chain.BVSDirectory).GetBVSInfo(core.C.Chain.BVSHash)
	if err != nil {
		panic(err)
	}

	delegation := api.NewDelegationImpl(client, core.C.Chain.DelegationManager)

	rewardsCoordinator := api.NewRewardsCoordinator(client)
	rewardsCoordinator.BindClient(core.C.Chain.RewardCoordinator)

	return &Uploader{
		chainIO:            client,
		delegation:         delegation,
		bvsContract:        txResp.BVSContract,
		rewardsCoordinator: rewardsCoordinator,
		pubKeyStr:          pubKeyStr,
	}
}

func (u *Uploader) Run() {
	ctx := context.Background()
	blockNum := u.getBlock(ctx)

	fmt.Println("latestBlock: ", blockNum)

	evtIndexer := indexer.NewEventIndexer(
		u.chainIO.GetClientCtx(),
		u.bvsContract,
		blockNum,
		[]string{"wasm-TaskResponded"},
		3,
		5)

	evtChain, err := evtIndexer.Run(ctx)
	if err != nil {
		panic(err)
	}

	fmt.Println("chain: ", evtChain)
	for evt := range evtChain {
		switch evt.EventType {
		case "wasm-TaskResponded":
			blockHeight := evt.BlockHeight
			txnHash := evt.TxHash
			taskId := fmt.Sprintf("%s", evt.AttrMap["taskId"])
			taskResult := fmt.Sprintf("%s", evt.AttrMap["result"])
			taskOperators := fmt.Sprintf("%s", evt.AttrMap["operators"])
			fmt.Printf("[TaskResponded] blockHeight: %d, txnHash: %s, taskId: %s, taskResult: %s, taskOperators: %s\n", blockHeight, txnHash, taskId, taskResult, taskOperators)
			u.calcReward(ctx, taskId, taskOperators)
		default:
			fmt.Printf("Unknown event type. evt: %+v\n", evt)
		}
	}
}

func (u *Uploader) getBlock(ctx context.Context) int64 {
	res, err := u.chainIO.QueryNodeStatus(ctx)
	if err != nil {
		panic(err)
	}
	latestBlock := res.SyncInfo.LatestBlockHeight
	return latestBlock
}

// calcReward calculate the reward for operators and stakers according to the given task ID and operators.
func (u *Uploader) calcReward(ctx context.Context, taskId string, operators string) {
	if core.S.RedisConn.SIsMember(ctx, core.PKSaveTask, taskId).Val() {
		fmt.Println("task already processed: ", taskId)
		return
	}

	core.S.RedisConn.SAdd(ctx, core.PKSaveTask, taskId)

	operatorList := strings.Split(operators, "&")
	operatorCnt := len(operatorList)
	rewardAmount := decimal.NewFromFloat(core.C.Reward.Amount)

	fmt.Println("rewardAmount: ", rewardAmount)
	operatorAmount := rewardAmount.Div(decimal.NewFromInt(int64(operatorCnt)))
	fmt.Println("operatorAmount: ", operatorAmount)

	submissionMap := make(map[string]*Submission)
	totalEarners := make([]Earner, 0)

	sAmount := operatorAmount.Mul(decimal.NewFromFloat(core.C.Reward.OperatorRatio)).Div(decimal.NewFromInt(100))
	oAmount := operatorAmount.Sub(sAmount)
	fmt.Println("sAmount: ", sAmount)
	fmt.Println("oAmount: ", oAmount)

	for _, operator := range operatorList {
		txnRsp, err := u.delegation.GetOperatorStakers(operator)
		if err != nil {
			fmt.Println("get operator stakers err: ", err)
		}
		fmt.Println("GetOperatorStakers txnRsp: ", txnRsp)

		totalStakerAmount := decimal.Zero
		earners := make([]Earner, 0)

		for _, staker := range txnRsp.StakersAndShares {
			stakerAmount := decimal.Zero
			earnerTokens := make([]*TokenAmount, 0)

			for _, strategy := range staker.SharesPerStrategy {
				strategyAmount, err := decimal.NewFromString(strategy[1])
				if err != nil {
					fmt.Println("parse float err: ", err)
					continue
				}
				stakerAmount = stakerAmount.Add(strategyAmount)
				strategyToken, err := u.rpcUnderlyingToken(strategy[0])
				if err != nil {
					fmt.Println("get strategy token err: ", err)
					continue
				}

				earnerTokens = append(earnerTokens, &TokenAmount{
					Strategy:     strategy[0],
					Token:        strategyToken,
					RewardAmount: "",
					StakeAmount:  strategyAmount,
				})
			}
			sort.Slice(earnerTokens, func(i, j int) bool {
				return earnerTokens[i].Token < earnerTokens[j].Token
			})

			earners = append(earners, Earner{
				Earner:           staker.Staker,
				TotalStakeAmount: stakerAmount,
				Tokens:           earnerTokens,
			})
			totalStakerAmount = totalStakerAmount.Add(stakerAmount)
		}

		fmt.Println("totalStakerAmount: ", totalStakerAmount)
		for _, s := range earners {
			if totalStakerAmount == decimal.Zero || s.TotalStakeAmount == decimal.Zero {
				continue
			}
			stakerReward := sAmount.Mul(s.TotalStakeAmount.Div(totalStakerAmount))

			for _, t := range s.Tokens {
				rewardAmount := stakerReward.Mul(t.StakeAmount).Div(s.TotalStakeAmount)
				if rewardAmount == decimal.Zero {
					continue
				}
				fmt.Println("rewardAmount: ", rewardAmount)
				t.RewardAmount = rewardAmount.Ceil().String()
				if a, ok := submissionMap[t.Strategy]; !ok {
					submissionMap[t.Strategy] = &Submission{
						Strategy: t.Strategy,
						Token:    t.Token,
						Amount:   rewardAmount,
					}
				} else {
					a.Amount = a.Amount.Add(rewardAmount)
				}
			}
		}
		operatorStrategyToken, err := u.rpcUnderlyingToken(core.C.Reward.OperatorStrategy)
		if err != nil {
			fmt.Println("get strategy token err: ", err)
			continue
		}
		if a, ok := submissionMap[core.C.Reward.OperatorStrategy]; !ok {
			submissionMap[core.C.Reward.OperatorStrategy] = &Submission{
				Strategy: core.C.Reward.OperatorStrategy,
				Token:    operatorStrategyToken,
				Amount:   oAmount,
			}
		} else {
			a.Amount = a.Amount.Add(oAmount)
		}
		operatorRewardAmount := oAmount.Ceil().String()
		earners = append(earners, Earner{
			Earner:           operator,
			TotalStakeAmount: oAmount,
			Tokens: []*TokenAmount{
				{
					Strategy:     core.C.Reward.OperatorStrategy,
					Token:        operatorStrategyToken,
					RewardAmount: operatorRewardAmount,
					StakeAmount:  oAmount,
				},
			},
		})
		fmt.Printf("earners: %+v\n", earners)
		totalEarners = append(totalEarners, earners...)
	}

	// update earner reward token amount
	var totalEarnerTokenAmount []EarnerTokenAmount
	for _, earner := range totalEarners {
		for _, token := range earner.Tokens {
			oldAmount, err := u.getEarnerTokenAmount(earner.Earner, token.Token)
			if err != nil {
				fmt.Println("rpc update earner amount err: ", err)
				return
			}
			rAmount, _ := decimal.NewFromString(token.RewardAmount)
			newAmount := oldAmount.Add(rAmount).Ceil().String()
			token.RewardAmount = newAmount
			earnerTokenAmount := EarnerTokenAmount{
				Earner: earner.Earner,
				Token:  token.Token,
				Amount: newAmount,
			}
			totalEarnerTokenAmount = append(totalEarnerTokenAmount, earnerTokenAmount)
		}
	}

	totalEarnersJSON, err := json.Marshal(totalEarners)
	if err != nil {
		fmt.Println("totalEarnersJSON json err: ", err)
	}
	fmt.Println("totalEarnersJSON: ", string(totalEarnersJSON))

	// submission
	submissions := make([]Submission, 0)
	for _, submission := range submissionMap {
		submissions = append(submissions, *submission)
	}
	submissionsJSON, err := json.Marshal(submissions)
	if err != nil {
		fmt.Println("submissionsJSON json err: ", err)
	}
	fmt.Println("submissionsJSON: ", string(submissionsJSON))

	if err := u.rpcSubmission(submissions); err != nil {
		fmt.Println("rpc submission err: ", err)
		return
	}

	// merkle tree
	rootHash, rewardEarner, err := u.merkleTree(totalEarners)
	if err != nil {
		fmt.Println("merkle tree err: ", err)
		return
	}

	if err := u.rpcSubmitHashRoot(rootHash, rewardEarner, totalEarnerTokenAmount); err != nil {
		fmt.Println("rpc root hash err: ", err)
		return
	}
}

//

// merkleTree builds a merkle tree from the given earners, and returns the hash of the root node.
// The merkle tree is built by first calculating the merkle tree of each earner's tokens, and
// then building a merkle tree from the hashes of the earner token merkle trees.
// If an error occurs in the process, an error is returned.
func (u *Uploader) merkleTree(earners []Earner) (string, []*RewardEarner, error) {
	earnerHashes := make([]string, 0)
	rewardEarner := make([]*RewardEarner, 0)

	for _, earner := range earners {
		rewardTokens := make([]*RewardToken, 0)
		for _, token := range earner.Tokens {
			rewardToken := &RewardToken{
				Token:  token.Token,
				Amount: token.RewardAmount,
			}
			rewardTokens = append(rewardTokens, rewardToken)
		}
		tokenHash := u.calcTokenLeafs(earner.Tokens)
		rewardEarner = append(rewardEarner, &RewardEarner{
			Earner:    earner.Earner,
			TokenHash: tokenHash,
			Tokens:    rewardTokens,
		})
		fmt.Printf("token hash: %s\n", tokenHash)
		earnerHash, err := u.rpcEarnerLeafHash(earner.Earner, tokenHash)
		if err != nil {
			fmt.Println("calc earner hash err: ", err)
			return "", rewardEarner, err
		}
		earnerHashes = append(earnerHashes, earnerHash)
	}

	fmt.Println("earn hashs: ", earnerHashes)
	if len(earnerHashes) == 1 {
		return earnerHashes[0], rewardEarner, nil
	}
	rootHash, err := u.rpcMerkleizeLeaves(earnerHashes)
	if err != nil {
		fmt.Println("merkleizeLeaves err: ", err)
		return "", rewardEarner, err
	}
	fmt.Println("earner rootHash: ", rootHash)

	return rootHash, rewardEarner, nil
}

// calcTokenLeafs calculates the merkle root hash of the given tokens.
// It does this by first calculating the merkle tree of the tokens, and
// then returning the hash of the root node.
func (u *Uploader) calcTokenLeafs(tokens []*TokenAmount) string {
	var hashes []string
	for _, token := range tokens {
		hash, err := u.rpcTokenHash(token)
		if err != nil {
			fmt.Println("calc token hash err: ", err)
			continue
		}
		hashes = append(hashes, hash)
	}
	fmt.Println("token leaf hashes: ", hashes)
	if len(hashes) == 1 {
		return hashes[0]
	}
	rootHash, err := u.rpcMerkleizeLeaves(hashes)
	if err != nil {
		fmt.Println("merkleizeLeaves err: ", err)
		return ""
	}
	fmt.Println("token root hash: ", rootHash)
	return rootHash
}

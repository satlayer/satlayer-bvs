package uploader

import (
	"context"
	"fmt"
	"math"
	"strconv"
	"strings"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/indexer"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"

	"github.com/satlayer/satlayer-bvs/examples/squaring/uploader/core"
)

type Uploader struct {
	bvsContract        string
	delegation         *api.DelegationManager
	chainIO            io.ChainIO
	rewardsCoordinator *api.RewardsCoordinator
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

	txResp, err := api.NewDirectory(client, core.C.Chain.BVSDirectory).BvsInfo(core.C.Chain.BVSHash)
	if err != nil {
		panic(err)
	}

	delegation := api.NewDelegationManager(client, core.C.Chain.DelegationManager)

	rewardsCoordinator := api.NewRewardsCoordinator(client)
	rewardsCoordinator.BindClient(core.C.Chain.RewardCoordinator)

	return &Uploader{
		chainIO:            client,
		delegation:         delegation,
		bvsContract:        txResp.BvsContract,
		rewardsCoordinator: rewardsCoordinator,
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
			taskId := evt.AttrMap["taskId"]
			taskResult := evt.AttrMap["result"]
			taskOperators := evt.AttrMap["operators"]
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

	operatorAmount := core.C.Reward.Amount / float64(operatorCnt)
	fmt.Println("operatorAmount: ", operatorAmount)

	submissionMap := make(map[string]*Submission)
	totalEarners := make([]Earner, 0)

	sAmount := operatorAmount * core.C.Reward.OperatorRatio / 100
	oAmount := operatorAmount - sAmount
	fmt.Println("sAmount: ", sAmount)
	fmt.Println("oAmount: ", oAmount)

	for _, operator := range operatorList {
		txnRsp, err := u.delegation.GetOperatorStakers(operator)
		if err != nil {
			fmt.Println("get operator stakers err: ", err)
		}
		fmt.Println("GetOperatorStakers txnRsp: ", txnRsp)

		totalStakerAmount := 0.0
		earners := make([]Earner, 0)

		for _, staker := range txnRsp.StakersAndShares {
			stakerAmount := 0.0
			earnerTokens := make([]*TokenAmount, 0)

			for _, strategy := range staker.SharesPerStrategy {
				amount, err := strconv.ParseUint(strategy[1], 10, 0)
				if err != nil {
					fmt.Println("parse float err: ", err)
					continue
				}

				strategyAmount := float64(amount)
				stakerAmount += strategyAmount
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

			earners = append(earners, Earner{
				Earner:           staker.Staker,
				TotalStakeAmount: stakerAmount,
				Tokens:           earnerTokens,
			})
			totalStakerAmount += stakerAmount
		}

		fmt.Println("totalStakerAmount: ", totalStakerAmount)
		for _, s := range earners {
			if totalStakerAmount == 0.0 || s.TotalStakeAmount == 0.0 {
				continue
			}

			stakerReward := sAmount * (s.TotalStakeAmount / totalStakerAmount)

			for _, t := range s.Tokens {
				rewardAmount := stakerReward * t.StakeAmount / s.TotalStakeAmount
				if rewardAmount == 0.0 {
					continue
				}
				fmt.Println("rewardAmount: ", rewardAmount)

				t.RewardAmount = strconv.FormatFloat(math.Floor(rewardAmount), 'f', -1, 64)
				if a, ok := submissionMap[t.Strategy]; !ok {
					submissionMap[t.Strategy] = &Submission{
						Strategy: t.Strategy,
						Token:    t.Token,
						Amount:   rewardAmount,
					}
				} else {
					a.Amount += rewardAmount
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
			a.Amount += oAmount
		}
		operatorRewardAmount := strconv.FormatFloat(math.Floor(oAmount), 'f', -1, 64)
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

	// submission
	submissions := make([]Submission, 0)
	for _, submission := range submissionMap {
		submissions = append(submissions, *submission)
		fmt.Printf("strategy: %s, token: %s, amount: %f\n", submission.Strategy, submission.Token, submission.Amount)
	}
	fmt.Printf("earners: %+v\n", totalEarners)

	if err := u.rpcSubmission(submissions); err != nil {
		fmt.Println("rpc submission err: ", err)
		return
	}

	// merkle tree
	rootHash, err := u.merkleTree(totalEarners)
	if err != nil {
		fmt.Println("merkle tree err: ", err)
		return
	}
	fmt.Printf("root Hash: %s\n", rootHash)

	if err := u.rpcSubmitHashRoot(rootHash); err != nil {
		fmt.Println("rpc root hash err: ", err)
		return
	}
}

// merkleTree builds a merkle tree from the given earners, and returns the hash of the root node.
// The merkle tree is built by first calculating the merkle tree of each earner's tokens, and
// then building a merkle tree from the hashes of the earner token merkle trees.
// If an error occurs in the process, an error is returned.
func (u *Uploader) merkleTree(earners []Earner) (string, error) {
	// calc earner token merkle tree
	earnerNodes := make([]*MerkleNode, 0)
	for _, earner := range earners {
		tokenHash := u.calcTokenLeafs(earner.Tokens)
		earnerHash, err := u.rpcEarnerLeafHash(earner.Earner, tokenHash)
		if err != nil {
			fmt.Println("calc earner hash err: ", err)
			return "", err
		}
		earnerNodes = append(earnerNodes, &MerkleNode{Hash: earnerHash})
	}

	root := u.calcMerkleTree(earnerNodes)
	return root.Hash, nil
}

// calcTokenLeafs calculates the merkle root hash of the given tokens.
// It does this by first calculating the merkle tree of the tokens, and
// then returning the hash of the root node.
func (u *Uploader) calcTokenLeafs(tokens []*TokenAmount) string {
	tokenNodes := make([]*MerkleNode, 0)
	for _, token := range tokens {
		hash, err := u.rpcTokenHash(token)
		if err != nil {
			fmt.Println("calc token hash err: ", err)
			continue
		}
		tokenNodes = append(tokenNodes, &MerkleNode{Hash: hash})
	}
	fmt.Println("tokenHashs: ", tokenNodes)
	root := u.calcMerkleTree(tokenNodes)
	return root.Hash
}

// calcMerkleTree calculates the merkle tree of the given nodes and returns the root node.
// The merkle tree is built by iterating through the nodes in pairs, and for each pair,
// calculating the root hash of the two nodes and appending it to the new level.
func (u *Uploader) calcMerkleTree(nodes []*MerkleNode) *MerkleNode {
	// calc merkle tree
	for len(nodes) > 1 {
		if len(nodes)%2 != 0 {
			nodes = append(nodes, nodes[len(nodes)-1])
		}

		var newLevel []*MerkleNode
		for i := 0; i < len(nodes); i += 2 {
			var left, right *MerkleNode
			left = nodes[i]
			right = nodes[i+1]
			leaves := []string{left.Hash, right.Hash}
			rootHash, err := u.rpcMerkleizeLeaves(leaves)
			if err != nil {
				fmt.Println("merkleizeLeaves err: ", err)
				continue
			}

			newNode := &MerkleNode{
				Left:  left,
				Right: right,
				Hash:  rootHash,
			}

			newLevel = append(newLevel, newNode)
		}
		nodes = newLevel
	}
	return nodes[0]
}

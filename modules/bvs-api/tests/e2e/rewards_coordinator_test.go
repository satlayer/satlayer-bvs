package e2e

import (
	"context"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"strings"
	"testing"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	apilogger "github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
)

const rewardsCoordinatorAddr = "bbn1xwpk5mrrrm7zsl606mhdj5lmtmegcu9c72ve7hyd7kf7n3v2jnrq2wgyxf"
const staker = "bbn17y9szawx0gsjcrycukr3kud36kfcclee7zwwvc"
const strategyAddress = "bbn14rruau4y52cqyag6d9pxa3rrwhhh9xu7egndpafu55ztd8dprj8s860s8w"
const token = "bbn1mx295r0mph0xvetqqcapsj4xxreg9mek7nlzhcacu4y0r83hhxfqu9mn0v"

type rewardsTestSuite struct {
	suite.Suite
	chainIO io.ChainIO
}

func (suite *rewardsTestSuite) SetupTest() {
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"
	homeDir := "../.babylon" // Please refer to the readme to obtain

	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "rewards")
	chainIO, err := io.NewChainIO(chainID, rpcURI, homeDir, "bbn", logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          2 * time.Second,
		ConfirmationTimeout:    15 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	suite.Require().NoError(err)
	suite.chainIO = chainIO
}

func (suite *rewardsTestSuite) Test_ExecuteRewardsCoordinator() {
	t := suite.T()
	keyName := "uploader"

	t.Logf("TestRewards")
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	rewardsCoordinator := api.NewRewardsCoordinator(chainIO)
	rewardsCoordinator.BindClient(rewardsCoordinatorAddr)

	const calcInterval = 86_400 // 1 day
	now := time.Now().Unix()
	startTime := now - now%calcInterval
	//t.Log("startTime:", fmt.Sprintf("%d", startTime), startTime%calcInterval)

	resp, err := rewardsCoordinator.CreateBVSRewardsSubmission(
		context.Background(),
		[]types.RewardsSubmission{{
			StrategiesAndMultipliers: []types.StrategyAndMultiplier{{
				Strategy:   strategyAddress,
				Multiplier: 1,
			}},
			Token:          token,
			Amount:         fmt.Sprintf("%d", 10),                 // need to convert to string type
			StartTimestamp: fmt.Sprintf("%d000000000", startTime), //need to convert to string type, unit is nano second
			Duration:       calcInterval,
		}})

	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	chainIO, err = suite.chainIO.SetupKeyring("wallet2", "test")
	assert.NoError(t, err)

	rewardsCoordinator = api.NewRewardsCoordinator(chainIO)
	rewardsCoordinator.BindClient(rewardsCoordinatorAddr)

	resp, err = rewardsCoordinator.SetRewardsForAllSubmitter(context.Background(), staker, true)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	chainIO, err = suite.chainIO.SetupKeyring("uploader", "test")
	assert.NoError(t, err)

	rewardsCoordinator = api.NewRewardsCoordinator(chainIO)
	rewardsCoordinator.BindClient(rewardsCoordinatorAddr)

	resp, err = rewardsCoordinator.CreateRewardsForAllSubmission(
		context.Background(),
		[]types.RewardsSubmission{{
			StrategiesAndMultipliers: []types.StrategyAndMultiplier{{
				Strategy:   strategyAddress,
				Multiplier: 1,
			}},
			Token:          token,
			Amount:         fmt.Sprintf("%d", 10),                 // need to convert to string type
			StartTimestamp: fmt.Sprintf("%d000000000", startTime), //need to convert to string type
			Duration:       calcInterval,
		}})

	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	chainIO, err = suite.chainIO.SetupKeyring("wallet2", "test")
	assert.NoError(t, err)

	rewardsCoordinator = api.NewRewardsCoordinator(chainIO)
	rewardsCoordinator.BindClient(rewardsCoordinatorAddr)

	resp, err = rewardsCoordinator.SetRewardsUpdater(context.Background(), staker)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)
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

func (suite *rewardsTestSuite) test_QueryRewardsCoordinator() {
	t := suite.T()
	keyName := "wallet1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	rewardsCoordinator := api.NewRewardsCoordinator(chainIO)
	rewardsCoordinator.BindClient(rewardsCoordinatorAddr)

	resp, err := rewardsCoordinator.OperatorCommissionBips("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x", "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.Error(t, err, "execute contract")
	assert.Nil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = rewardsCoordinator.GetDistributionRootsLength()
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = rewardsCoordinator.GetCurrentDistributionRoot()
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = rewardsCoordinator.GetDistributionRootAtIndex("1")
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = rewardsCoordinator.GetCurrentClaimableDistributionRoot()
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	tokenRootHash := []byte{157, 182, 190, 113, 122, 8, 74, 164, 14, 216, 104, 83, 161, 117, 200, 187, 2, 25, 18, 169, 181, 254, 114, 62, 226, 208, 14, 25, 176, 189, 118, 122}
	rootHash := make([]string, len(tokenRootHash))

	for i, b := range tokenRootHash {
		rootHash[i] = hex.EncodeToString([]byte{b})
	}

	resp, err = rewardsCoordinator.GetRootIndexFromHash(strings.Join(rootHash, ""))
	assert.Error(t, err, "execute contract")
	assert.Nil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = rewardsCoordinator.CalculateDomainSeparator("sat-bbn-testnet1", token)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)
}

func (suite *rewardsTestSuite) Test_SubmitRoot() {
	t := suite.T()
	keyName := "uploader"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	rewardsCoordinator := api.NewRewardsCoordinator(chainIO)
	rewardsCoordinator.BindClient(rewardsCoordinatorAddr)

	earnerLeaf, tokenRootHash, leafB := calculateEarnerLeaf(rewardsCoordinator, t)
	t.Logf("earner leaf:%+v", bytesToString(earnerLeaf))

	earnerLeaf1, tokenRootHash1, _ := calculateEarnerLeaf(rewardsCoordinator, t)
	t.Logf("earner leaf1:%+v", bytesToString(earnerLeaf1))

	resp, err := rewardsCoordinator.MerkleizeLeaves([]string{
		base64.StdEncoding.EncodeToString(earnerLeaf),
		base64.StdEncoding.EncodeToString(earnerLeaf1),
	})
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v, %+v, %+v, %+v", resp, tokenRootHash, leafB, tokenRootHash1)

	var rootHashResponse MerkleizeLeavesResponse
	err = json.Unmarshal(resp.Data, &rootHashResponse)
	assert.NoError(t, err, "execute contract")

	rootHash := rootHashResponse.RootHashBinary
	t.Logf("root hash:%+v", rootHash)

	timestamp := time.Now().Unix() - 3600
	res, err := rewardsCoordinator.SubmitRoot(context.Background(), base64.StdEncoding.EncodeToString(rootHash), uint64(timestamp))
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, res, "response nil")
	t.Logf("resp:%+v", res)

	/*res, err = rewardsCoordinator.DisableRoot(context.Background(),3)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, res, "response nil")
	t.Logf("resp:%+v", res)*/
}

func (suite *rewardsTestSuite) test_CheckClaim() {
	t := suite.T()
	keyName := "wallet1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	rewardsCoordinator := api.NewRewardsCoordinator(chainIO)
	rewardsCoordinator.BindClient(rewardsCoordinatorAddr)

	earnerLeaf1 := []byte{219, 9, 161, 135, 125, 102, 17, 167, 215, 74, 251, 185, 74, 116, 4, 92, 77, 131, 254, 124, 32, 111, 16, 125, 221, 212, 50, 124, 91, 169, 109, 36}
	tokenRootHash := []byte{157, 182, 190, 113, 122, 8, 74, 164, 14, 216, 104, 83, 161, 117, 200, 187, 2, 25, 18, 169, 181, 254, 114, 62, 226, 208, 14, 25, 176, 189, 118, 122}
	leafB := []byte{206, 115, 226, 107, 25, 126, 7, 79, 220, 253, 12, 111, 1, 16, 210, 149, 171, 32, 239, 250, 170, 1, 225, 240, 187, 214, 142, 240, 36, 15, 155, 97}
	parentCD := []byte{127, 135, 181, 129, 134, 0, 46, 147, 5, 33, 229, 55, 20, 214, 171, 170, 192, 53, 195, 132, 255, 84, 26, 58, 251, 12, 31, 137, 253, 223, 171, 243}

	rootIndex := uint32(0)
	earnerIndex := uint32(0)
	earnerTreeProof := bytesToUints(earnerLeaf1)

	leaf := types.ExecuteEarnerTreeMerkleLeaf{
		Earner:          staker,
		EarnerTokenRoot: base64.StdEncoding.EncodeToString(tokenRootHash),
	}

	tokenIndices := []uint32{0}

	tokenTreeProofs := [][]uint16{append(bytesToUints(leafB), bytesToUints(parentCD)...)} // parent node hash for C, D

	tokenLeaves := []types.TokenTreeMerkleLeaf{
		{
			Token:              token,
			CumulativeEarnings: "15",
		},
	}

	// earnerTreeProof, leaf, tokenTreeProofs
	claim :=
		types.ExeuteRewardsMerkleClaim{
			RootIndex:       rootIndex,
			EarnerIndex:     earnerIndex,
			EarnerTreeProof: earnerTreeProof,
			EarnerLeaf:      leaf,
			TokenIndices:    tokenIndices,
			TokenTreeProofs: tokenTreeProofs,
			TokenLeaves:     tokenLeaves,
		}
	t.Logf("claim:%+v", claim)

	checkResp, err := rewardsCoordinator.CheckClaim(claim)

	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, checkResp, "response nil")
	t.Logf("resp:%+v", checkResp)
}

func (suite *rewardsTestSuite) test_ProcessClaim() {
	t := suite.T()
	keyName := "wallet1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	rewardsCoordinator := api.NewRewardsCoordinator(chainIO)
	rewardsCoordinator.BindClient(rewardsCoordinatorAddr)

	earnerLeaf1 := []byte{193, 236, 171, 13, 54, 199, 205, 10, 46, 215, 61, 182, 187, 231, 93, 170, 79, 252, 86, 54, 113, 168, 1, 43, 25, 96, 174, 173, 3, 88, 168, 122}
	tokenRootHash := []byte{226, 185, 241, 197, 117, 165, 165, 145, 104, 161, 171, 134, 48, 163, 31, 74, 225, 159, 66, 82, 123, 59, 225, 60, 46, 218, 55, 192, 124, 52, 61, 177}
	leafB := []byte{210, 69, 106, 161, 254, 2, 52, 3, 108, 143, 253, 152, 113, 19, 132, 27, 24, 82, 101, 150, 109, 94, 102, 107, 205, 14, 7, 15, 79, 41, 89, 43}
	parentCD := []byte{103, 51, 76, 183, 37, 230, 19, 197, 35, 70, 76, 43, 118, 87, 119, 67, 86, 241, 16, 36, 93, 129, 24, 173, 51, 94, 223, 165, 116, 18, 214, 112}
	rootIndex := uint32(2)
	earnerIndex := uint32(0)
	earnerTreeProof := bytesToUints(earnerLeaf1)

	leaf := types.ExecuteEarnerTreeMerkleLeaf{
		Earner:          staker,
		EarnerTokenRoot: base64.StdEncoding.EncodeToString(tokenRootHash),
	}

	tokenIndices := []uint32{0}

	tokenTreeProofs := [][]uint16{append(bytesToUints(leafB), bytesToUints(parentCD)...)} // parent node hash for C, D

	tokenLeaves := []types.TokenTreeMerkleLeaf{
		{
			Token:              token,
			CumulativeEarnings: "30",
		},
	}

	// earnerTreeProof, leaf, tokenTreeProofs
	claim :=
		types.ExeuteRewardsMerkleClaim{
			RootIndex:       rootIndex,
			EarnerIndex:     earnerIndex,
			EarnerTreeProof: earnerTreeProof,
			EarnerLeaf:      leaf,
			TokenIndices:    tokenIndices,
			TokenTreeProofs: tokenTreeProofs,
			TokenLeaves:     tokenLeaves,
		}
	t.Logf("claim:%+v", claim)

	checkResp, err := rewardsCoordinator.ProcessClaim(context.Background(), claim, staker)

	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, checkResp, "response nil")
	t.Logf("resp:%+v", checkResp)
}

func (suite *rewardsTestSuite) Test_IsRewardsUpdater() {
	t := suite.T()
	keyName := "wallet1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	rewardsCoordinator := api.NewRewardsCoordinator(chainIO)
	rewardsCoordinator.BindClient(rewardsCoordinatorAddr)
	result, err := rewardsCoordinator.IsRewardsUpdater("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.NoError(t, err, "execute contract")
	t.Logf("result:%v", result)
}

func bytesToUints(arr []byte) []uint16 {
	int8Array := make([]uint16, len(arr))

	for i, b := range arr {
		int8Array[i] = uint16(b)
	}

	return int8Array
}

func calculateParentNode(rewardsCoordinator api.RewardsCoordinator, t *testing.T) ([]byte, []byte, []byte) {
	resp, err := rewardsCoordinator.CalculateTokenLeafHash(token, "30")
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	//t.Logf("resp:%+v", resp)

	var hashResponse HashResponse
	var hashBytes []byte
	err = json.Unmarshal(resp.Data, &hashResponse)
	assert.NoError(t, err, "execute contract")

	hashBytes = hashResponse.HashBinary
	t.Logf("hash:%+v", bytesToString(hashBytes))

	resp, err = rewardsCoordinator.CalculateTokenLeafHash(token, "30")
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	//t.Logf("resp:%+v", resp)

	var hashBytes1 []byte
	err = json.Unmarshal(resp.Data, &hashResponse)
	assert.NoError(t, err, "execute contract")

	hashBytes1 = hashResponse.HashBinary
	t.Logf("hash2:%+v", bytesToString(hashBytes1))

	leaf := base64.StdEncoding.EncodeToString(hashBytes)
	leaf1 := base64.StdEncoding.EncodeToString(hashBytes1)

	resp, err = rewardsCoordinator.MerkleizeLeaves([]string{leaf, leaf1})
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")

	var rootHashResponse MerkleizeLeavesResponse
	err = json.Unmarshal(resp.Data, &rootHashResponse)
	assert.NoError(t, err, "execute contract")

	t.Logf("parent node hash:%+v", bytesToString(rootHashResponse.RootHashBinary))

	parentNode := rootHashResponse.RootHashBinary
	return parentNode, hashBytes, hashBytes1
}

func calculateEarnerLeaf(rewardsCoordinator api.RewardsCoordinator, t *testing.T) ([]byte, []byte, []byte) {
	parentNode, _, leafB := calculateParentNode(rewardsCoordinator, t)
	parentNode1, _, _ := calculateParentNode(rewardsCoordinator, t)

	resp, err := rewardsCoordinator.MerkleizeLeaves([]string{
		base64.StdEncoding.EncodeToString(parentNode), base64.StdEncoding.EncodeToString(parentNode1)})
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")

	var rootHashResponse MerkleizeLeavesResponse
	err = json.Unmarshal(resp.Data, &rootHashResponse)
	assert.NoError(t, err, "execute contract")

	rootHash := rootHashResponse.RootHashBinary
	t.Logf("root hash:%+v", bytesToString(rootHash))

	resp, err = rewardsCoordinator.CalculateEarnerLeafHash(staker, base64.StdEncoding.EncodeToString(rootHash))
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	//t.Logf("earner leaf:%+v", resp.Data)

	var earnerLeafHashResponse EarnerLeafHashResponse
	var earnerLeaf []byte

	err = json.Unmarshal(resp.Data, &earnerLeafHashResponse)
	assert.NoError(t, err, "execute contract")
	t.Logf("parsed earner leaf:%+v", bytesToString(earnerLeaf))

	earnerLeaf = earnerLeafHashResponse.RootHashBinary
	return earnerLeaf, rootHash, leafB
}

func bytesToString(content []byte) string {
	strSlice := make([]string, len(content))
	for i, b := range content {
		strSlice[i] = fmt.Sprintf("%d", b)
	}

	return strings.Join(strSlice, ",")
}

func TestRewardsTestSuite(t *testing.T) {
	suite.Run(t, new(rewardsTestSuite))
}

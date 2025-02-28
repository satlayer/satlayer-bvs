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

	rewardscoordinator "github.com/satlayer/satlayer-bvs/bvs-cw/rewards-coordinator"

	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/satlayer/satlayer-bvs/babylond/cw20"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
)

type rewardsTestSuite struct {
	suite.Suite
	chainIO                io.ChainIO
	rewardsCoordinatorAddr string
	tokenAddr              string
	strategyManagerAddr    string
	container              *babylond.BabylonContainer
	caller                 string
}

func (suite *rewardsTestSuite) SetupSuite() {
	container := babylond.Run(context.Background())
	suite.chainIO = container.NewChainIO("../.babylon")
	suite.container = container
	suite.caller = "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"

	// Fund Caller
	container.ImportPrivKey("rewards-coordinator:initial_owner", "E5DBC50CB04311A2A5C3C0E0258D396E962F64C6C2F758458FFB677D7F0C0E94")
	container.FundAddressUbbn(suite.caller, 1e8)
	tAddr := container.GenerateAddress("test-address").String()
	deployer := &bvs.Deployer{BabylonContainer: container}
	registry := deployer.DeployRegistry(nil)

	token := cw20.DeployCw20(container, cw20.InstantiateMsg{
		Decimals: 6,
		InitialBalances: []cw20.Cw20Coin{
			{
				Address: suite.caller,
				Amount:  "1000000000",
			},
		},
		Mint: &cw20.MinterResponse{
			Minter: suite.caller,
		},
		Name:   "Test Token",
		Symbol: "TEST",
	})
	suite.tokenAddr = token.Address

	status, err := container.ClientCtx.Client.Status(context.Background())
	suite.Require().NoError(err)
	blockTime := status.SyncInfo.LatestBlockTime.Second()

	strategyManager := deployer.DeployStrategyManager(registry.Address, tAddr, tAddr, "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	rewardsCoordinator := deployer.DeployRewardsCoordinator(
		registry.Address,
		// Test Vector taken from: bvs-rewards-coordinator/src/contract.rs
		60,     // 1 minute
		86_400, // 1 day
		int64(blockTime)/86_400*86_400,
		10*86_400, // 10 days
		5*86_400,  // 5 days
		30*86_400, // 30 days
	)
	suite.rewardsCoordinatorAddr = rewardsCoordinator.Address
	suite.strategyManagerAddr = strategyManager.Address

	chainIO, err := suite.chainIO.SetupKeyring("caller", "test")
	suite.NoError(err)
	rewardsApi := api.NewRewardsCoordinator(chainIO)
	rewardsApi.BindClient(rewardsCoordinator.Address)
	res, err := rewardsApi.SetRouting(context.Background(),
		tAddr,
		suite.strategyManagerAddr,
	)
	suite.NoError(err)
	suite.Equal(uint32(0), res.TxResult.Code)

	res, err = rewardsApi.SetRewardsUpdater(context.Background(), suite.caller)
	suite.NoError(err)
	suite.Equal(uint32(0), res.TxResult.Code)
}

func (suite *rewardsTestSuite) TearDownSuite() {
	suite.Require().NoError(suite.container.Terminate(context.Background()))
}

func (suite *rewardsTestSuite) Test_SetRouting() {
	chainIO, err := suite.chainIO.SetupKeyring("caller", "test")
	suite.NoError(err)

	rewardsCoordinator := api.NewRewardsCoordinator(chainIO)
	rewardsCoordinator.BindClient(suite.rewardsCoordinatorAddr)

	res, err := rewardsCoordinator.SetRouting(context.Background(),
		suite.container.GenerateAddress("delegation-manager").String(),
		suite.strategyManagerAddr,
	)

	suite.NoError(err)
	suite.Equal(uint32(0), res.TxResult.Code)
}

func (suite *rewardsTestSuite) Test_ExecuteRewardsCoordinator() {
	t := suite.T()
	keyName := "caller"

	t.Logf("TestRewards")
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	rewardsCoordinator := api.NewRewardsCoordinator(chainIO)
	rewardsCoordinator.BindClient(suite.rewardsCoordinatorAddr)

	const calcInterval = 86_400 // 1 day
	now := time.Now().Unix()
	startTime := now - now%calcInterval
	//t.Log("startTime:", fmt.Sprintf("%d", startTime), startTime%calcInterval)

	strategyManagerAddr := suite.container.GenerateAddress("strategy")

	smApi := api.NewStrategyManager(chainIO)
	smApi.BindClient(suite.strategyManagerAddr)
	tx, err := smApi.AddStrategiesToWhitelist(context.Background(), []string{strategyManagerAddr.String()})
	assert.NoError(t, err, "execute contract")
	suite.Equal(uint32(0), tx.TxResult.Code)

	allowanceMsg := cw20.ExecuteMsg{
		IncreaseAllowance: &cw20.IncreaseAllowance{
			Spender: suite.rewardsCoordinatorAddr,
			Amount:  "1000000000",
		},
	}

	bytes, err := allowanceMsg.Marshal()
	assert.NoError(t, err, "marshal allowanceMsg")

	res, err := chainIO.ExecuteContract(types.ExecuteOptions{
		ContractAddr:  suite.tokenAddr,
		ExecuteMsg:    bytes,
		Funds:         "",
		GasAdjustment: 1.2,
		GasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		Gas:           1000000,
		Memo:          "Allowance",
		Simulate:      true,
	})
	assert.NoError(t, err, "execute contract")
	suite.Equal(uint32(0), res.Code)

	resp, err := rewardsCoordinator.CreateBVSRewardsSubmission(
		context.Background(),
		[]rewardscoordinator.RewardsSubmission{{
			StrategiesAndMultipliers: []rewardscoordinator.StrategyAndMultiplier{{
				Strategy:   strategyManagerAddr.String(),
				Multiplier: 1,
			}},
			Token:          suite.tokenAddr,
			Amount:         fmt.Sprintf("%d", 10),                 // need to convert to string type
			StartTimestamp: fmt.Sprintf("%d000000000", startTime), //need to convert to string type, unit is nano second
			Duration:       calcInterval,
		}})

	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = rewardsCoordinator.SetRewardsForAllSubmitter(context.Background(), suite.caller, true)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = rewardsCoordinator.CreateRewardsForAllSubmission(
		context.Background(),
		[]rewardscoordinator.RewardsSubmission{{
			StrategiesAndMultipliers: []rewardscoordinator.StrategyAndMultiplier{{
				Strategy:   strategyManagerAddr.String(),
				Multiplier: 1,
			}},
			Token:          suite.tokenAddr,
			Amount:         fmt.Sprintf("%d", 10),                 // need to convert to string type
			StartTimestamp: fmt.Sprintf("%d000000000", startTime), //need to convert to string type
			Duration:       calcInterval,
		}})

	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = rewardsCoordinator.SetRewardsUpdater(context.Background(), suite.caller)
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
	rewardsCoordinator.BindClient(suite.rewardsCoordinatorAddr)

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
}

func (suite *rewardsTestSuite) Test_SubmitRoot() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	rewardsCoordinator := api.NewRewardsCoordinator(chainIO)
	rewardsCoordinator.BindClient(suite.rewardsCoordinatorAddr)

	earnerLeaf, tokenRootHash, leafB := suite.calculateEarnerLeaf(rewardsCoordinator, t)
	t.Logf("earner leaf:%+v", bytesToString(earnerLeaf))

	earnerLeaf1, tokenRootHash1, _ := suite.calculateEarnerLeaf(rewardsCoordinator, t)
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
	res, err := rewardsCoordinator.SubmitRoot(context.Background(), base64.StdEncoding.EncodeToString(rootHash), timestamp)
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
	rewardsCoordinator.BindClient(suite.rewardsCoordinatorAddr)

	earnerLeaf1 := []byte{219, 9, 161, 135, 125, 102, 17, 167, 215, 74, 251, 185, 74, 116, 4, 92, 77, 131, 254, 124, 32, 111, 16, 125, 221, 212, 50, 124, 91, 169, 109, 36}
	tokenRootHash := []byte{157, 182, 190, 113, 122, 8, 74, 164, 14, 216, 104, 83, 161, 117, 200, 187, 2, 25, 18, 169, 181, 254, 114, 62, 226, 208, 14, 25, 176, 189, 118, 122}
	leafB := []byte{206, 115, 226, 107, 25, 126, 7, 79, 220, 253, 12, 111, 1, 16, 210, 149, 171, 32, 239, 250, 170, 1, 225, 240, 187, 214, 142, 240, 36, 15, 155, 97}
	parentCD := []byte{127, 135, 181, 129, 134, 0, 46, 147, 5, 33, 229, 55, 20, 214, 171, 170, 192, 53, 195, 132, 255, 84, 26, 58, 251, 12, 31, 137, 253, 223, 171, 243}

	rootIndex := int64(0)
	earnerIndex := int64(0)
	earnerTreeProof := bytesToUints(earnerLeaf1)

	leaf := rewardscoordinator.FluffyExecuteEarnerTreeMerkleLeaf{
		Earner:          suite.caller,
		EarnerTokenRoot: base64.StdEncoding.EncodeToString(tokenRootHash),
	}

	tokenIndices := []int64{0}

	tokenTreeProofs := [][]int64{append(bytesToUints(leafB), bytesToUints(parentCD)...)} // parent node hash for C, D

	tokenLeaves := []rewardscoordinator.FluffyTokenTreeMerkleLeaf{
		{
			Token:              suite.tokenAddr,
			CumulativeEarnings: "15",
		},
	}

	// earnerTreeProof, leaf, tokenTreeProofs
	claim := rewardscoordinator.CheckClaimClaim{
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
	rewardsCoordinator.BindClient(suite.rewardsCoordinatorAddr)

	earnerLeaf1 := []byte{193, 236, 171, 13, 54, 199, 205, 10, 46, 215, 61, 182, 187, 231, 93, 170, 79, 252, 86, 54, 113, 168, 1, 43, 25, 96, 174, 173, 3, 88, 168, 122}
	tokenRootHash := []byte{226, 185, 241, 197, 117, 165, 165, 145, 104, 161, 171, 134, 48, 163, 31, 74, 225, 159, 66, 82, 123, 59, 225, 60, 46, 218, 55, 192, 124, 52, 61, 177}
	leafB := []byte{210, 69, 106, 161, 254, 2, 52, 3, 108, 143, 253, 152, 113, 19, 132, 27, 24, 82, 101, 150, 109, 94, 102, 107, 205, 14, 7, 15, 79, 41, 89, 43}
	parentCD := []byte{103, 51, 76, 183, 37, 230, 19, 197, 35, 70, 76, 43, 118, 87, 119, 67, 86, 241, 16, 36, 93, 129, 24, 173, 51, 94, 223, 165, 116, 18, 214, 112}
	rootIndex := int64(2)
	earnerIndex := int64(0)
	earnerTreeProof := bytesToUints(earnerLeaf1)

	leaf := rewardscoordinator.PurpleExecuteEarnerTreeMerkleLeaf{
		Earner:          suite.caller,
		EarnerTokenRoot: base64.StdEncoding.EncodeToString(tokenRootHash),
	}

	tokenIndices := []int64{0}

	tokenTreeProofs := [][]int64{append(bytesToUints(leafB), bytesToUints(parentCD)...)} // parent node hash for C, D

	tokenLeaves := []rewardscoordinator.PurpleTokenTreeMerkleLeaf{
		{
			Token:              suite.tokenAddr,
			CumulativeEarnings: "30",
		},
	}

	// earnerTreeProof, leaf, tokenTreeProofs
	claim := rewardscoordinator.ProcessClaimClaim{
		RootIndex:       rootIndex,
		EarnerIndex:     earnerIndex,
		EarnerTreeProof: earnerTreeProof,
		EarnerLeaf:      leaf,
		TokenIndices:    tokenIndices,
		TokenTreeProofs: tokenTreeProofs,
		TokenLeaves:     tokenLeaves,
	}
	t.Logf("claim:%+v", claim)

	checkResp, err := rewardsCoordinator.ProcessClaim(context.Background(), claim, suite.caller)

	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, checkResp, "response nil")
	t.Logf("resp:%+v", checkResp)
}

func bytesToUints(arr []byte) []int64 {
	int8Array := make([]int64, len(arr))

	for i, b := range arr {
		int8Array[i] = int64(b)
	}

	return int8Array
}

func calculateParentNode(
	tokenAddr string,
	rewardsCoordinator *api.RewardsCoordinator, t *testing.T) ([]byte, []byte, []byte) {
	resp, err := rewardsCoordinator.CalculateTokenLeafHash(tokenAddr, "30")
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	//t.Logf("resp:%+v", resp)

	var hashResponse HashResponse
	var hashBytes []byte
	err = json.Unmarshal(resp.Data, &hashResponse)
	assert.NoError(t, err, "execute contract")

	hashBytes = hashResponse.HashBinary
	t.Logf("hash:%+v", bytesToString(hashBytes))

	resp, err = rewardsCoordinator.CalculateTokenLeafHash(tokenAddr, "30")
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

func (suite *rewardsTestSuite) calculateEarnerLeaf(rewardsCoordinator *api.RewardsCoordinator, t *testing.T) ([]byte, []byte, []byte) {
	parentNode, _, leafB := calculateParentNode(suite.tokenAddr, rewardsCoordinator, t)
	parentNode1, _, _ := calculateParentNode(suite.tokenAddr, rewardsCoordinator, t)

	resp, err := rewardsCoordinator.MerkleizeLeaves([]string{
		base64.StdEncoding.EncodeToString(parentNode), base64.StdEncoding.EncodeToString(parentNode1)})
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")

	var rootHashResponse MerkleizeLeavesResponse
	err = json.Unmarshal(resp.Data, &rootHashResponse)
	assert.NoError(t, err, "execute contract")

	rootHash := rootHashResponse.RootHashBinary
	t.Logf("root hash:%+v", bytesToString(rootHash))

	resp, err = rewardsCoordinator.CalculateEarnerLeafHash(suite.caller, base64.StdEncoding.EncodeToString(rootHash))
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

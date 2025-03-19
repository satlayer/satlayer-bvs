package cosmwasmapi

import (
	"context"
	"strconv"
	"testing"

	abci "github.com/cometbft/cometbft/abci/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/satlayer/satlayer-bvs/cosmwasm-schema/pauser"
	"github.com/stretchr/testify/suite"
)

type ClientTestSuite struct {
	suite.Suite
	container *babylond.BabylonContainer
	pauser    *bvs.Contract[pauser.InstantiateMsg]
}

func (s *ClientTestSuite) SetupSuite() {
	s.container = babylond.Run(context.Background())
	s.container.ClientCtx = s.container.ClientCtx.WithFromAddress(s.container.GenerateAddress("owner")).WithFromName("owner").WithFrom("owner")
}

// deploy contract has to be done before each test
// prevents unit tests data from interfering with each other
func (s *ClientTestSuite) SetupTest() {
	deployer := &bvs.Deployer{BabylonContainer: s.container}
	s.pauser = deployer.DeployPauser(nil)
}

func (s *ClientTestSuite) TearDownSuite() {
	s.Require().NoError(s.container.Terminate(context.Background()))
}

func TestClient(t *testing.T) {
	suite.Run(t, new(ClientTestSuite))
}

func (s *ClientTestSuite) TestQuery() {
	contract := s.container.GenerateAddress("contract")
	queryMsg := pauser.QueryMsg{
		IsPaused: &pauser.IsPaused{
			C: contract.String(),
			M: "Deposit",
		},
	}

	response, err := Query[pauser.IsPausedResponse](
		s.container.ClientCtx,
		context.Background(),
		s.pauser.Address,
		queryMsg,
	)
	s.Require().NoError(err)
	s.Equal(pauser.IsPausedResponse(0), response)
}

func (s *ClientTestSuite) TestExecute() {
	contract := s.container.GenerateAddress("contract")
	owner := s.container.GenerateAddress("owner")
	_ = s.container.FundAddressUbbn(owner.String(), 1000)

	executeMsg := pauser.ExecuteMsg{
		Pause: &pauser.Pause{},
	}

	executeOptions := NewBroadcastOptions(
		s.pauser.Address,
		executeMsg,
	).WithGasPrice("0.002ubbn")

	response, err := Execute(
		s.container.ClientCtx,
		context.Background(),
		owner.String(),
		executeOptions,
	)

	s.Require().NoError(err)

	expectedEvent := sdktypes.Event{
		Type: "wasm",
		Attributes: []abci.EventAttribute{
			{Key: "_contract_address", Value: s.pauser.Address},
			{Key: "method", Value: "pause"},
			{Key: "sender", Value: owner.String()},
			{Key: "msg_index", Value: strconv.Itoa(0)},
		},
	}

	// Compare the specific event
	actualEvent := response.TxResult.Events[9]
	s.Equal(expectedEvent.Type, actualEvent.Type)

	// Compare attributes individually
	for i, attr := range expectedEvent.Attributes {
		s.Equal(attr.Key, actualEvent.Attributes[i].Key)
		s.Equal(attr.Value, actualEvent.Attributes[i].Value)
	}

	// assert that contract is actually paused
	queryMsg := pauser.QueryMsg{
		IsPaused: &pauser.IsPaused{
			C: contract.String(),
			M: "Deposit",
		},
	}
	isPausedResponse, err := Query[pauser.IsPausedResponse](
		s.container.ClientCtx,
		context.Background(),
		s.pauser.Address,
		queryMsg,
	)
	s.Require().NoError(err)
	s.Equal(pauser.IsPausedResponse(1), isPausedResponse)
}

func (s *ClientTestSuite) TestWaitForTx() {
	receiver := s.container.GenerateAddress("receiver")

	// create a TX by sending some ubbn to the receiver
	tx := s.container.FundAddressUbbn(receiver.String(), 10000)

	txHash := tx.Hash.String()
	txRes, err := WaitForTx(
		s.container.ClientCtx,
		context.Background(),
		txHash,
	)

	s.Require().NoError(err)
	s.Equal(txHash, txRes.Hash.String())
}

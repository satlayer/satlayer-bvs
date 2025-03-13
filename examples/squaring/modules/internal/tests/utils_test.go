package tests

import (
	"context"
	"testing"

	squaringcontract "github.com/satlayer/satlayer-bvs/examples/squaring/squaring-contract"
	"github.com/stretchr/testify/suite"
)

type SquaringTestSuit struct {
	TestSuite
}

func (s *SquaringTestSuit) SetupSuite() {
	s.TestSuite.SetupSuite("../../.babylond", "wallet1", "820d902159777d247dda5922d3e2669477e2a2059a03f7ace61a32981e85848e")
}

// entrypoint for the test suite
func TestSquaring(t *testing.T) {
	suite.Run(t, new(SquaringTestSuit))
}

func (s *SquaringTestSuit) TestExecuteSquaring() {
	squaringContract := s.DeploySquaringContract()

	bvsSquaring := squaringcontract.New(s.ChainIO)
	bvsSquaring.BindClient(squaringContract.Address)

	operator := s.Babylond.GenerateAddress("operator")

	resp, err := bvsSquaring.CreateNewTask(context.Background(), 10)
	s.NoError(err, "error creating task")
	s.NotNil(resp, "create task response nil")

	resp, err = bvsSquaring.RespondToTask(context.Background(), 10, 100, operator.String())
	s.NoError(err, "error responding to task")
	s.NotNil(resp, "respond task response nil")
}

func (s *SquaringTestSuit) TestQuerySquaring() {
	squaringContract := s.DeploySquaringContract()

	bvsSquaring := squaringcontract.New(s.ChainIO)
	bvsSquaring.BindClient(squaringContract.Address)

	// create first task
	resp, err := bvsSquaring.CreateNewTask(s.Ctx, 20)
	s.NoError(err, "error creating task")
	s.NotNil(resp, "create task response nil")

	// query first task
	res, err := bvsSquaring.GetTaskInput(1)
	s.NoError(err, "query contract")
	s.NotNil(res, "response nil")
	s.T().Logf("resp:%+v", res)

	// query first task result (not found)
	res, err = bvsSquaring.GetTaskResult(1)
	s.Error(err, "query contract")

	// query latest task ID
	res, err = bvsSquaring.GetLatestTaskID()
	s.NoError(err, "query contract")
	s.NotNil(res, "response nil")
	s.T().Logf("resp:%+v", res)
}

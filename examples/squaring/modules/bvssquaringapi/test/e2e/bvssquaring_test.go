package e2e

import (
	"context"
	"testing"

	"github.com/satlayer/satlayer-bvs/examples/squaring/internal/tests"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/examples/squaring/bvssquaringapi"
)

type SquaringTestSuit struct {
	tests.TestSuite
}

func (suite *SquaringTestSuit) SetupSuite() {
	suite.TestSuite.SetupSuite("../../../.babylond", "wallet1")

	suite.Babylond.FundAddressUbbn("bbn1huw8yau3aqdsp9lr2f85v5plfd46tu026wylaj", 1e8)
}

// entrypoint for the test suite
func TestSquaring(t *testing.T) {
	suite.Run(t, new(SquaringTestSuit))
}

func (suite *SquaringTestSuit) TestExecuteSquaring() {
	t := suite.T()

	squaringContract := suite.DeploySquaringContract()

	bvsSquaring := bvssquaringapi.NewBVSSquaring(suite.ChainIO)
	bvsSquaring.BindClient(squaringContract.Address)

	operator := suite.Babylond.GenerateAddress("operator")

	resp, err := bvsSquaring.CreateNewTask(context.Background(), 10)
	assert.NoError(t, err, "error creating task")
	assert.NotNil(t, resp, "create task response nil")

	resp, err = bvsSquaring.RespondToTask(context.Background(), 10, 100, operator.String())
	assert.NoError(t, err, "error responding to task")
	assert.NotNil(t, resp, "respond task response nil")
}

func (suite *SquaringTestSuit) TestQuerySquaring() {
	t := suite.T()

	squaringContract := suite.DeploySquaringContract()

	bvsSquaring := bvssquaringapi.NewBVSSquaring(suite.ChainIO)
	bvsSquaring.BindClient(squaringContract.Address)

	// create first task
	resp, err := bvsSquaring.CreateNewTask(suite.Ctx, 10)
	assert.NoError(t, err, "error creating task")
	assert.NotNil(t, resp, "create task response nil")

	// query first task
	res, err := bvsSquaring.GetTaskInput(1)
	assert.NoError(t, err, "query contract")
	assert.NotNil(t, res, "response nil")
	t.Logf("resp:%+v", res)

	// query first task result
	res, err = bvsSquaring.GetTaskResult(1)
	assert.NoError(t, err, "query contract")
	assert.NotNil(t, res, "response nil")
	t.Logf("resp:%+v", res)

	// query latest task ID
	res, err = bvsSquaring.GetLatestTaskID()
	assert.NoError(t, err, "query contract")
	assert.NotNil(t, res, "response nil")
	t.Logf("resp:%+v", res)
}

package babylond

import (
	"context"
	"github.com/stretchr/testify/suite"
	"os"
	"testing"
)

type WasmTestSuite struct {
	suite.Suite
	Container *BabylonContainer
}

func (s *WasmTestSuite) SetupSuite() {
	container, err := Run(context.Background())
	s.Require().NoError(err)
	s.Container = container
}

func (s *WasmTestSuite) TearDownSuite() {
	s.Require().NoError(s.Container.Container.Terminate(context.Background()))
}

func TestWasm(t *testing.T) {
	suite.Run(t, new(WasmTestSuite))
}

func (s *WasmTestSuite) TestWasm() {
	data, err := os.ReadFile("cw_bvs_driver.wasm")
	s.NoError(err)

	ctx := context.Background()
	res, err := s.Container.StoreWasmCode(ctx, data, "genesis")
	s.NoError(err)
	s.Equal(uint32(0), res.TxResult.Code)

	codeId, err := GetCodeId(res)
	s.Equal(uint64(1), codeId)

	json := `{"initial_owner": "bbn1lmnc4gcvcu5dexa8p6vv2e6qkas5lu2r2nwlnv"}`
	res, err = s.Container.InitWasmCode(ctx, codeId, []byte(json), "driver", "genesis")
	s.NoError(err)
	s.Equal(uint32(0), res.TxResult.Code)
}

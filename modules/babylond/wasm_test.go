package babylond

import (
	"context"
	"os"
	"testing"

	"github.com/stretchr/testify/suite"
)

type WasmTestSuite struct {
	suite.Suite
	Container *BabylonContainer
}

func (s *WasmTestSuite) SetupSuite() {
	s.Container = Run(context.Background())
}

func (s *WasmTestSuite) TearDownSuite() {
	s.Require().NoError(s.Container.Container.Terminate(context.Background()))
}

func TestWasm(t *testing.T) {
	suite.Run(t, new(WasmTestSuite))
}

// TestWasm tests the wasm module using a simple contract that increments a counter.
// Test contract is in https://github.com/fuxingloh/cw-test
func (s *WasmTestSuite) TestWasm() {
	data, err := os.ReadFile("wasm_test.wasm")
	s.NoError(err)

	res, err := s.Container.StoreWasmCode(data, "genesis")
	s.NoError(err)
	s.Equal(uint32(0), res.TxResult.Code)

	codeId, err := GetCodeId(res)
	s.Equal(uint64(1), codeId)

	json := `{"count": 10}`
	res, err = s.Container.InitWasmCode(codeId, []byte(json), "example", "genesis")
	s.NoError(err)
	s.Equal(uint32(0), res.TxResult.Code)

	addr, err := GetContractAddress(res)
	s.NoError(err)
	s.NotEmpty(addr)
}

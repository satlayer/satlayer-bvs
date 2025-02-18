package babylond

import (
	"encoding/json"

	bvscw "github.com/satlayer/satlayer-bvs/bvs-cw"
	statebank "github.com/satlayer/satlayer-bvs/bvs-cw/state-bank"
	strategybase "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-base"
)

type Deployed[T interface{}] struct {
	DeployedWasmContract
	InstantiateMsg T
}

func deployCrate[T interface{}](c *BabylonContainer, crate string, initMsg T, label string) *Deployed[T] {
	wasmByteCode, err := bvscw.ReadWasmFile(crate)
	if err != nil {
		panic(err)
	}

	initBytes, err := json.Marshal(initMsg)
	if err != nil {
		panic(err)
	}

	contract, err := c.StoreAndInitWasm(wasmByteCode, initBytes, label, "genesis")
	if err != nil {
		panic(err)
	}
	return &Deployed[T]{
		DeployedWasmContract: *contract,
		InstantiateMsg:       initMsg,
	}
}

func (c *BabylonContainer) DeployStateBank() *Deployed[statebank.InstantiateMsg] {
	initMsg := statebank.InstantiateMsg{
		InitialOwner: c.GenerateAddress("state-bank:initial_owner").String(),
	}
	return deployCrate(c, "bvs-state-bank", initMsg, "BVS State Bank")
}

func (c *BabylonContainer) DeployStrategyBase(underlyingToken, strategyManager string) *Deployed[strategybase.InstantiateMsg] {
	initMsg := strategybase.InstantiateMsg{
		InitialOwner:        c.GenerateAddress("strategy-base:initial_owner").String(),
		InitialPausedStatus: 0,
		Pauser:              c.GenerateAddress("strategy-base:pauser").String(),
		Unpauser:            c.GenerateAddress("strategy-base:unpauser").String(),
		StrategyManager:     strategyManager,
		UnderlyingToken:     underlyingToken,
	}

	return deployCrate(c, "bvs-strategy-base", initMsg, "BVS Strategy Base")
}

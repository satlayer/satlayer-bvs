package babylond

import (
	bvscw "github.com/satlayer/satlayer-bvs/bvs-cw"
	statebank "github.com/satlayer/satlayer-bvs/bvs-cw/state-bank"
)

func (c *BabylonContainer) deployCrate(crate string, initMsg []byte, label string) (*DeployedWasmContract, error) {
	wasmByteCode, err := bvscw.ReadWasmFile(crate)
	if err != nil {
		panic(err)
	}

	return c.StoreAndInitWasm(wasmByteCode, initMsg, label, "genesis")
}

type DeployedStateBank struct {
	DeployedWasmContract
	Owner string
}

func (c *BabylonContainer) DeployStateBank() *DeployedStateBank {
	initialOwner := c.GenerateAddress("state-bank:initial_owner").String()
	initMsg := statebank.InstantiateMsg{InitialOwner: initialOwner}
	initBytes, err := initMsg.Marshal()
	if err != nil {
		panic(err)
	}

	contract, err := c.deployCrate("bvs-state-bank", initBytes, "BVS State Bank")
	if err != nil {
		panic(err)
	}
	return &DeployedStateBank{
		DeployedWasmContract: *contract,
		Owner:                initialOwner,
	}
}

package bvs

import (
	"encoding/json"

	"github.com/satlayer/satlayer-bvs/babylond"

	"github.com/satlayer/satlayer-bvs/cosmwasm-schema/pauser"
)

type Contract[T interface{}] struct {
	babylond.DeployedWasmContract
	InstantiateMsg T
}

type Deployer struct {
	*babylond.BabylonContainer
}

func deployCrate[T interface{}](deployer *Deployer, crate string, initMsg T, label string) *Contract[T] {
	wasmByteCode, err := ReadArtifact(crate)
	if err != nil {
		panic(err)
	}

	initBytes, err := json.Marshal(initMsg)
	if err != nil {
		panic(err)
	}

	contract, err := deployer.StoreAndInitWasm(wasmByteCode, initBytes, label, "genesis")
	if err != nil {
		panic(err)
	}
	return &Contract[T]{
		DeployedWasmContract: *contract,
		InstantiateMsg:       initMsg,
	}
}

// TODO(fuxingloh): implement Deployer.DeployAll()

func (d *Deployer) DeployPauser(
	initMsg *pauser.InstantiateMsg,
) *Contract[pauser.InstantiateMsg] {
	if initMsg == nil {
		initMsg = &pauser.InstantiateMsg{
			InitialPaused: false,
			Owner:         d.GenerateAddress("owner").String(),
		}
	}

	return deployCrate(d, "bvs-pauser", *initMsg, "BVS Pauser")
}

package bvs

import (
	"encoding/json"

	"github.com/satlayer/satlayer-bvs/babylond"

	"github.com/satlayer/satlayer-bvs/cosmwasm-schema/pauser"
	"github.com/satlayer/satlayer-bvs/cosmwasm-schema/registry"
)

type Contract[T interface{}] struct {
	babylond.DeployedWasmContract
	InstantiateMsg T
}

type Deployer struct {
	*babylond.BabylonContainer
	Contracts map[string]string
}

func NewDeployer(container *babylond.BabylonContainer) *Deployer {
	return &Deployer{
		BabylonContainer: container,
		Contracts:        make(map[string]string),
	}
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
	deployer.Contracts[crate] = contract.Address
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

func (d *Deployer) DeployRegistry(
	initMsg *registry.InstantiateMsg,
) *Contract[registry.InstantiateMsg] {
	if initMsg == nil {
		initMsg = &registry.InstantiateMsg{
			Owner:  d.GenerateAddress("owner").String(),
			Pauser: d.Contracts["bvs-pauser"],
		}
	}

	return deployCrate(d, "bvs-registry", *initMsg, "BVS Registry")
}

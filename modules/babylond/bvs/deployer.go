package bvs

import (
	"encoding/json"

	"github.com/satlayer/satlayer-bvs/babylond"

	delegationmanager "github.com/satlayer/satlayer-bvs/cosmwasm-schema/delegation-manager"
	"github.com/satlayer/satlayer-bvs/cosmwasm-schema/directory"
	"github.com/satlayer/satlayer-bvs/cosmwasm-schema/pauser"
	rewardscoordinator "github.com/satlayer/satlayer-bvs/cosmwasm-schema/rewards-coordinator"
	slashmanager "github.com/satlayer/satlayer-bvs/cosmwasm-schema/slash-manager"
	strategybase "github.com/satlayer/satlayer-bvs/cosmwasm-schema/strategy-base"
	strategymanager "github.com/satlayer/satlayer-bvs/cosmwasm-schema/strategy-manager"
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

// TODO(fuxingloh): implement Deployer.Deploy()

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

func (d *Deployer) DeploySlashManager(
	pauser string,
) *Contract[slashmanager.InstantiateMsg] {
	initMsg := slashmanager.InstantiateMsg{
		Owner:  d.GenerateAddress("owner").String(),
		Pauser: pauser,
	}

	return deployCrate(d, "bvs-slash-manager", initMsg, "BVS Slash Manager")
}

func (d *Deployer) DeployStrategyManager(
	pauser string,
) *Contract[strategymanager.InstantiateMsg] {
	initMsg := strategymanager.InstantiateMsg{
		Owner:  d.GenerateAddress("owner").String(),
		Pauser: pauser,
	}

	return deployCrate(d, "bvs-strategy-manager", initMsg, "BVS Strategy Manager")
}

func (d *Deployer) DeployDelegationManager(
	pauser string,
	minWithdrawalDelayBlocks int64,
	strategies []string,
	withdrawalDelayBlocks []int64,
) *Contract[delegationmanager.InstantiateMsg] {
	initMsg := delegationmanager.InstantiateMsg{
		Owner:                    d.GenerateAddress("owner").String(),
		Pauser:                   pauser,
		MinWithdrawalDelayBlocks: minWithdrawalDelayBlocks,
		Strategies:               strategies,
		WithdrawalDelayBlocks:    withdrawalDelayBlocks,
	}

	return deployCrate(d, "bvs-delegation-manager", initMsg, "BVS Delegation Manager")
}

func (d *Deployer) DeployDirectory(
	pauser string,
) *Contract[directory.InstantiateMsg] {
	initMsg := directory.InstantiateMsg{
		Owner:  d.GenerateAddress("owner").String(),
		Pauser: pauser,
	}

	return deployCrate(d, "bvs-directory", initMsg, "BVS Directory")
}

func (d *Deployer) DeployRewardsCoordinator(
	pauser string,
	activationDelay int64,
	calculationIntervalSeconds int64,
	genesisRewardsTimestamp int64,
	maxFutureLength int64,
	maxRetroactiveLength int64,
	maxRewardsDuration int64,
) *Contract[rewardscoordinator.InstantiateMsg] {
	initMsg := rewardscoordinator.InstantiateMsg{
		Owner:                      d.GenerateAddress("owner").String(),
		Pauser:                     pauser,
		ActivationDelay:            activationDelay,
		CalculationIntervalSeconds: calculationIntervalSeconds,
		GenesisRewardsTimestamp:    genesisRewardsTimestamp,
		MaxFutureLength:            maxFutureLength,
		MaxRetroactiveLength:       maxRetroactiveLength,
		MaxRewardsDuration:         maxRewardsDuration,
	}

	return deployCrate(d, "bvs-rewards-coordinator", initMsg, "BVS Rewards Coordinator")
}

func (d *Deployer) DeployStrategyBase(
	pauser string,
	underlyingToken string,
	strategyManager string,
) *Contract[strategybase.InstantiateMsg] {
	initMsg := strategybase.InstantiateMsg{
		Owner:           d.GenerateAddress("owner").String(),
		Pauser:          pauser,
		StrategyManager: strategyManager,
		UnderlyingToken: underlyingToken,
	}

	return deployCrate(d, "bvs-strategy-base", initMsg, "BVS Strategy Base")
}

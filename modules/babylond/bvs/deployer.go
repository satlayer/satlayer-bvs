package bvs

import (
	"encoding/json"

	"github.com/satlayer/satlayer-bvs/babylond"

	delegationmanager "github.com/satlayer/satlayer-bvs/bvs-cw/delegation-manager"
	"github.com/satlayer/satlayer-bvs/bvs-cw/directory"
	"github.com/satlayer/satlayer-bvs/bvs-cw/registry"
	rewardscoordinator "github.com/satlayer/satlayer-bvs/bvs-cw/rewards-coordinator"
	slashmanager "github.com/satlayer/satlayer-bvs/bvs-cw/slash-manager"
	strategybase "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-base"
	strategymanager "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-manager"
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

func (d *Deployer) DeployRegistry(
	initMsg *registry.InstantiateMsg,
) *Contract[registry.InstantiateMsg] {
	if initMsg == nil {
		initMsg = &registry.InstantiateMsg{
			InitialPaused: false,
			Owner:         d.GenerateAddress("registry:owner").String(),
		}
	}

	return deployCrate(d, "bvs-registry", *initMsg, "BVS Registry")
}

func (d *Deployer) DeploySlashManager(
	registry string,
) *Contract[slashmanager.InstantiateMsg] {
	initMsg := slashmanager.InstantiateMsg{
		Owner:    d.GenerateAddress("slash-manager:initial_owner").String(),
		Registry: registry,
	}

	return deployCrate(d, "bvs-slash-manager", initMsg, "BVS Slash Manager")
}

func (d *Deployer) DeployStrategyManager(
	registry string,
	delegationManager string,
	slashManager string,
	initialStrategyWhitelister string,
) *Contract[strategymanager.InstantiateMsg] {
	initMsg := strategymanager.InstantiateMsg{
		Owner:                      d.GenerateAddress("strategy-manager:initial_owner").String(),
		Registry:                   registry,
		DelegationManager:          delegationManager,
		SlashManager:               slashManager,
		InitialStrategyWhitelister: initialStrategyWhitelister,
	}

	return deployCrate(d, "bvs-strategy-manager", initMsg, "BVS Strategy Manager")
}

func (d *Deployer) DeployDelegationManager(
	registry string,
	minWithdrawalDelayBlocks int64,
	strategies []string,
	withdrawalDelayBlocks []int64,
) *Contract[delegationmanager.InstantiateMsg] {
	initMsg := delegationmanager.InstantiateMsg{
		Owner:                    d.GenerateAddress("delegation-manager:initial_owner").String(),
		Registry:                 registry,
		MinWithdrawalDelayBlocks: minWithdrawalDelayBlocks,
		Strategies:               strategies,
		WithdrawalDelayBlocks:    withdrawalDelayBlocks,
	}

	return deployCrate(d, "bvs-delegation-manager", initMsg, "BVS Delegation Manager")
}

func (d *Deployer) DeployDirectory(
	registry string,
) *Contract[directory.InstantiateMsg] {
	initMsg := directory.InstantiateMsg{
		Owner:    d.GenerateAddress("directory:initial_owner").String(),
		Registry: registry,
	}

	return deployCrate(d, "bvs-directory", initMsg, "BVS Directory")
}

func (d *Deployer) DeployRewardsCoordinator(
	registry string,
	activationDelay int64,
	calculationIntervalSeconds int64,
	genesisRewardsTimestamp int64,
	maxFutureLength int64,
	maxRetroactiveLength int64,
	maxRewardsDuration int64,
) *Contract[rewardscoordinator.InstantiateMsg] {
	initMsg := rewardscoordinator.InstantiateMsg{
		Owner:                      d.GenerateAddress("rewards-coordinator:initial_owner").String(),
		Registry:                   registry,
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
	registry string,
	underlyingToken string,
	strategyManager string,
) *Contract[strategybase.InstantiateMsg] {
	initMsg := strategybase.InstantiateMsg{
		Owner:           d.GenerateAddress("strategy-base:initial_owner").String(),
		Registry:        registry,
		StrategyManager: strategyManager,
		UnderlyingToken: underlyingToken,
	}

	return deployCrate(d, "bvs-strategy-base", initMsg, "BVS Strategy Base")
}

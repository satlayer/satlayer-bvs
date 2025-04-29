package utils

import (
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/types"
)

const (
	maxPostgreSQLParams = 65535
)

func SplitWASMInstantiateContracts(contracts []types.WASMInstantiateContract, paramsNumber int) [][]types.WASMInstantiateContract {
	maxBalancesPerSlice := maxPostgreSQLParams / paramsNumber
	slices := make([][]types.WASMInstantiateContract, len(contracts)/maxBalancesPerSlice+1)

	sliceIndex := 0
	for index, contract := range contracts {
		slices[sliceIndex] = append(slices[sliceIndex], contract)

		if index > 0 && index%(maxBalancesPerSlice-1) == 0 {
			sliceIndex++
		}
	}

	return slices
}

func SplitWASMExecuteContracts(executeContracts []types.WASMExecuteContract, paramsNumber int) [][]types.WASMExecuteContract {
	maxBalancesPerSlice := maxPostgreSQLParams / paramsNumber
	slices := make([][]types.WASMExecuteContract, len(executeContracts)/maxBalancesPerSlice+1)

	sliceIndex := 0
	for index, executeContract := range executeContracts {
		slices[sliceIndex] = append(slices[sliceIndex], executeContract)

		if index > 0 && index%(maxBalancesPerSlice-1) == 0 {
			sliceIndex++
		}
	}

	return slices
}

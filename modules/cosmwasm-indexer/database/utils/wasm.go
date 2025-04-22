package utils

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/types"
)

const (
	maxPostgreSQLParams = 65535
)

func SplitWasmContracts(contracts []types.WasmContract, paramsNumber int) [][]types.WasmContract {
	maxBalancesPerSlice := maxPostgreSQLParams / paramsNumber
	slices := make([][]types.WasmContract, len(contracts)/maxBalancesPerSlice+1)

	sliceIndex := 0
	for index, contract := range contracts {
		slices[sliceIndex] = append(slices[sliceIndex], contract)

		if index > 0 && index%(maxBalancesPerSlice-1) == 0 {
			sliceIndex++
		}
	}

	return slices
}

func SplitWasmExecuteContracts(executeContracts []types.WasmExecuteContract, paramsNumber int) [][]types.WasmExecuteContract {
	maxBalancesPerSlice := maxPostgreSQLParams / paramsNumber
	slices := make([][]types.WasmExecuteContract, len(executeContracts)/maxBalancesPerSlice+1)

	sliceIndex := 0
	for index, executeContract := range executeContracts {
		slices[sliceIndex] = append(slices[sliceIndex], executeContract)

		if index > 0 && index%(maxBalancesPerSlice-1) == 0 {
			sliceIndex++
		}
	}

	return slices
}

// FindEventByType returns the event with the given type
func FindEventByType(events sdk.StringEvents, eventType string) (sdk.StringEvent, bool) {
	for _, event := range events {
		if event.Type == eventType {
			return event, true
		}
	}
	return sdk.StringEvent{}, false
}

// FindAttributeByKey returns the attribute with the given key
func FindAttributeByKey(event sdk.StringEvent, key string) (sdk.Attribute, bool) {
	for _, attribute := range event.Attributes {
		if attribute.Key == key {
			return attribute, true
		}
	}
	return sdk.Attribute{}, false
}

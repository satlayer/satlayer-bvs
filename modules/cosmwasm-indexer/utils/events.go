package utils

import (
	"encoding/json"
	"strings"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// FindWASMEvents returns the WASM events
func FindWASMEvents(events sdk.StringEvents) sdk.StringEvents {
	var wasmEvents sdk.StringEvents
	for _, event := range events {
		if event.Type == wasmtypes.WasmModuleEventType {
			wasmEvents = append(wasmEvents, event)
			continue
		}
	}
	return wasmEvents
}

// FindCustomWASMEvents returns the custom WASM events
func FindCustomWASMEvents(events sdk.StringEvents) sdk.StringEvents {
	var customWASMEvents sdk.StringEvents
	for _, event := range events {
		if strings.Contains(event.Type, wasmtypes.CustomContractEventPrefix) {
			customWASMEvents = append(customWASMEvents, event)
			continue
		}
	}
	return customWASMEvents
}

// ExtractStringEvent extracts key-value pairs from StringEvent "attributes" field.
func ExtractStringEvents(events sdk.StringEvents) ([]byte, error) {
	resultArray := make([]map[string]any, len(events))
	for i, event := range events {
		newAttrs := make([]map[string]string, 0, len(event.Attributes))
		for _, attr := range event.Attributes {
			newAttrs = append(newAttrs, map[string]string{
				attr.Key: attr.Value,
			})
		}

		result := map[string]any{
			"type":       event.Type,
			"attributes": newAttrs,
		}
		resultArray[i] = result
	}

	return json.Marshal(resultArray)
}

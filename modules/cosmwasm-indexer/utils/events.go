package utils

import (
	"encoding/json"
	"strings"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// FindEventByType returns the event with the given type
func FindEventByType(events sdk.StringEvents, eventType string) (sdk.StringEvent, bool) {
	for _, event := range events {
		if event.Type == eventType {
			return event, true
		}
	}
	return sdk.StringEvent{}, false
}

// FindCustomWASMEvent returns the event with the custom WASM type
func FindCustomWASMEvent(events sdk.StringEvents) (sdk.StringEvent, bool) {
	for _, event := range events {
		if strings.Contains(event.Type, wasmtypes.CustomContractEventPrefix) {
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

// ExtractStringEvent extracts key-value pairs from StringEvent "attributes" field.
func ExtractStringEvent(event sdk.StringEvent) ([]byte, error) {
	newAttrs := make([]map[string]string, 0, len(event.Attributes))
	for _, attr := range event.Attributes {
		newAttrs = append(newAttrs, map[string]string{
			attr.Key: attr.Value,
		})
	}

	result := map[string]interface{}{
		"type":       event.Type,
		"attributes": newAttrs,
	}

	return json.Marshal(result)
}

package utils

import (
	"strings"

	sdktypes "github.com/cosmos/cosmos-sdk/types"
)

// IsValidContractAddr checks the given address is valid.
func IsValidContractAddr(addr string, prefix string) bool {
	if !strings.HasPrefix(addr, prefix) {
		return false
	}

	_, err := sdktypes.AccAddressFromBech32(addr)
	return err == nil
}

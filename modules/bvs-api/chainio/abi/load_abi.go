package abi

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/ethereum/go-ethereum/accounts/abi"
)

var abiCache = make(map[string]*abi.ABI)

func GetContractABI(abiPath string, contractName string) (*abi.ABI, error) {
	if cachedABI, ok := abiCache[contractName]; ok {
		return cachedABI, nil
	}
	filePath := fmt.Sprintf("%s/%s.json", abiPath, contractName)
	s, _ := filepath.Abs(filePath)
	parsedABI, err := loadABI(s)
	if err != nil {
		return nil, err
	}
	abiCache[contractName] = parsedABI
	return parsedABI, nil
}

func loadABI(filePath string) (*abi.ABI, error) {
	abiFile, err := os.Open(filePath)
	if err != nil {
		return nil, err
	}
	defer abiFile.Close()

	parsedABI, err := abi.JSON(abiFile)
	if err != nil {
		return nil, err
	}

	return &parsedABI, nil
}

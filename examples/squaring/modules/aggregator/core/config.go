package core

import (
	"path/filepath"
	"runtime"

	"github.com/BurntSushi/toml"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
)

var C Config
var L logger.Logger
var S Store

func InitConfig(overrideConfig Config) {
	_, currentFile, _, ok := runtime.Caller(0)
	if !ok {
		panic("cannot get current file")
	}

	// get env.file path
	configDir := filepath.Dir(currentFile)
	envFilePath := filepath.Join(configDir, "../env.toml")
	if _, err := toml.DecodeFile(envFilePath, &C); err != nil {
		panic(err)
	}

	// override config
	UpdateConfig(overrideConfig)

	// init logger
	L = logger.NewELKLogger(C.Chain.BvsContract)
	initStore(&C.Database)
}

func UpdateConfig(overrideConfig Config) {
	if overrideConfig.Database.RedisHost != "" {
		C.Database.RedisHost = overrideConfig.Database.RedisHost
	}
	if overrideConfig.Owner.KeyName != "" {
		C.Owner.KeyName = overrideConfig.Owner.KeyName
	}
	if overrideConfig.Owner.KeyDir != "" {
		C.Owner.KeyDir = overrideConfig.Owner.KeyDir
	}
	if overrideConfig.Owner.Bech32Prefix != "" {
		C.Owner.Bech32Prefix = overrideConfig.Owner.Bech32Prefix
	}
	if overrideConfig.Owner.KeyringBackend != "" {
		C.Owner.KeyringBackend = overrideConfig.Owner.KeyringBackend
	}
	if overrideConfig.Chain.ID != "" {
		C.Chain.ID = overrideConfig.Chain.ID
	}
	if overrideConfig.Chain.RPC != "" {
		C.Chain.RPC = overrideConfig.Chain.RPC
	}
	if overrideConfig.Chain.BvsDirectory != "" {
		C.Chain.BvsDirectory = overrideConfig.Chain.BvsDirectory
	}
	if overrideConfig.Chain.BvsContract != "" {
		C.Chain.BvsContract = overrideConfig.Chain.BvsContract
	}
}

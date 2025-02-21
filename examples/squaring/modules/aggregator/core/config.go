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

	// TODO: add more override and move to a separate function
	if overrideConfig.Database.RedisHost != "" {
		C.Database.RedisHost = overrideConfig.Database.RedisHost
	}
	if overrideConfig.Owner.KeyName != "" {
		C.Owner.KeyName = overrideConfig.Owner.KeyName
	}
	if overrideConfig.Owner.KeyDir != "" {
		C.Owner.KeyDir = overrideConfig.Owner.KeyDir
	}

	// init logger
	L = logger.NewELKLogger(C.Chain.BvsHash)
	initStore(&C.Database)
}

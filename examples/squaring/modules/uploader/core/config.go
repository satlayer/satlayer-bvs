package core

import (
	"fmt"
	"path/filepath"
	"runtime"

	"github.com/BurntSushi/toml"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
)

var C Config
var L logger.Logger
var S Store

// InitConfig Initializes the package by loading configuration from env.toml and setting up the logger.
//
// No parameters.
// No return values.
func InitConfig() {
	// load env.toml file
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
	fmt.Printf("C: %+v", C)
	// init logger
	L = logger.NewELKLogger(C.Chain.BVSHash)
	initStore(&C.Database)
}

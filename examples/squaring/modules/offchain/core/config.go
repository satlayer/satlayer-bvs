package core

import (
	"fmt"

	"github.com/BurntSushi/toml"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
)

var C Config
var L logger.Logger

// InitConfig initializes the package by loading configuration from env.toml and setting up the logger.
//
// No parameters.
// No return values.
func InitConfig() {
	// load env.toml file
	if _, err := toml.DecodeFile("env.toml", &C); err != nil {
		panic(err)
	}
	fmt.Println("C: ", C)
	L = logger.NewELKLogger(C.Chain.BVSContract)
}

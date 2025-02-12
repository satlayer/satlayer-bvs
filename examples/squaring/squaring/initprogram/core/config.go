package core

import (
	"fmt"
	"github.com/BurntSushi/toml"
)

var C Config

func InitConfig() {
	if _, err := toml.DecodeFile("env.toml", &C); err != nil {
		panic(err)
	}
	fmt.Printf("C: %+v", C)
}

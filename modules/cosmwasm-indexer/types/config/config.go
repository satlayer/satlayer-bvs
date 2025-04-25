package config

import (
	initcmd "github.com/forbole/juno/v6/cmd/init"
	junoconfig "github.com/forbole/juno/v6/types/config"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"

	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/modules/wasm"
)

// Config defines all necessary juno configuration parameters.
type Config struct {
	JunoConfig junoconfig.Config `yaml:"-,inline"`
	WasmConfig wasm.Config       `yaml:"wasms"`
}

// NewConfig returns a new Config instance
func NewConfig(junoCfg junoconfig.Config, wasmCfg wasm.Config) Config {
	return Config{
		JunoConfig: junoCfg,
		WasmConfig: wasmCfg,
	}
}

// Creator represents a configuration creator
func Creator(_ *cobra.Command) initcmd.WritableConfig {
	return NewConfig(junoconfig.DefaultConfig(), wasm.DefaultConfig())
}

func (c Config) GetBytes() ([]byte, error) {
	return yaml.Marshal(&c)
}

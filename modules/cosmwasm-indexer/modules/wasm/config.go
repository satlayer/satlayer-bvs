package wasm

import (
	"gopkg.in/yaml.v3"
)

// Config specify contract addresses that want to listen.
// key is contract name, value is contract address.
type Config map[string]string

// NewConfig returns wasm module config instance.
func NewConfig(contracts map[string]string) Config {
	return contracts
}

// DefaultConfig returns the default wasm module config.
func DefaultConfig() Config {
	contracts := map[string]string{}
	return NewConfig(contracts)
}

// ParseConfig parses wasm config from yaml config file
func ParseConfig(bz []byte) (Config, error) {
	type T struct {
		Config *Config `yaml:"wasm"`
	}

	var cfg T
	err := yaml.Unmarshal(bz, &cfg)
	return *cfg.Config, err
}

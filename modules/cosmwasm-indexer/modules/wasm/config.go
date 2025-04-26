package wasm

import (
	"fmt"

	"gopkg.in/yaml.v3"
)

// Config specify contract addresses that want to listen.
// The key of Contracts is contract address, value is contract label name.
// CodeID is used to fill the specified code id.
type Config struct {
	Contracts map[string]string `yaml:"contracts"`
	CodeID    CodeIDSlice       `yaml:"code_id"`
}

// NewConfig returns wasm module config instance.
func NewConfig(contracts map[string]string, codeID []uint64) Config {
	return Config{
		Contracts: contracts,
		CodeID:    codeID,
	}
}

// DefaultConfig returns the default wasm module config.
func DefaultConfig() Config {
	contracts := map[string]string{
		"contract_address": "contract_label",
	}
	codeID := []uint64{0}
	return NewConfig(contracts, codeID)
}

// ParseConfig parses wasm config from yaml config file
func ParseConfig(bz []byte) (*Config, error) {
	type T struct {
		Config *Config `yaml:"wasm"`
	}

	var cfg T
	err := yaml.Unmarshal(bz, &cfg)
	return cfg.Config, err
}

// CodeIDSlice is alias for uint64 slice
type CodeIDSlice []uint64

func (c CodeIDSlice) MarshalYAML() (any, error) {
	return &yaml.Node{
		Kind: yaml.SequenceNode,
		Tag:  "!!seq",
		Content: func() []*yaml.Node {
			nodes := make([]*yaml.Node, len(c))
			for i, v := range c {
				nodes[i] = &yaml.Node{
					Kind:  yaml.ScalarNode,
					Tag:   "!!int",
					Value: fmt.Sprintf("%d", v),
				}
			}
			return nodes
		}(),
		Style: yaml.FlowStyle,
	}, nil
}

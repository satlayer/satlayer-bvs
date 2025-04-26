package wasm

import (
	"fmt"

	"gopkg.in/yaml.v3"
)

// Config specify contract addresses that want to listen.
// The key of Contracts is contract address, value is contract label name.
// CodeIDs is used to fill the specified code id.
type Config struct {
	Contracts map[string]string `yaml:"contracts"`
	CodeIDs   Uint64Slice       `yaml:"code_ids"`
}

// NewConfig returns wasm module config instance.
func NewConfig(contracts map[string]string, codeIDs []uint64) Config {
	return Config{
		Contracts: contracts,
		CodeIDs:   codeIDs,
	}
}

// DefaultConfig returns the default wasm module config.
func DefaultConfig() Config {
	contracts := map[string]string{
		"contract_address": "contract_label_name",
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

// Uint64Slice is used for YAML marshal/unmarshal.
type Uint64Slice []uint64

func (u Uint64Slice) MarshalYAML() (any, error) {
	return &yaml.Node{
		Kind: yaml.SequenceNode,
		Tag:  "!!seq",
		Content: func() []*yaml.Node {
			nodes := make([]*yaml.Node, len(u))
			for i, v := range u {
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

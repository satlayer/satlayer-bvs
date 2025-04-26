package wasm

import (
	"fmt"

	"gopkg.in/yaml.v3"
)

type FlowSlice []uint64

// Config specify contract addresses that want to listen.
// key is contract address, value is contract label name.
type Config struct {
	Contracts map[string]string `yaml:"contracts"`
	CodeID    FlowSlice         `yaml:"code_id"`
}

func (f FlowSlice) MarshalYAML() (any, error) {
	return &yaml.Node{
		Kind: yaml.SequenceNode,
		Tag:  "!!seq",
		Content: func() []*yaml.Node {
			nodes := make([]*yaml.Node, len(f))
			for i, v := range f {
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
		"ccc": "cw20",
	}
	codeID := []uint64{1, 2, 3}
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

package config

import (
	initcmd "github.com/forbole/juno/v6/cmd/init"
	junoconfig "github.com/forbole/juno/v6/types/config"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"
)

// Config defines all necessary juno configuration parameters.
type Config struct {
	JunoConfig      junoconfig.Config `yaml:"-,inline"`
	ContractsConfig ContractsConfig   `yaml:"contracts"`
}

// NewConfig returns a new Config instance
func NewConfig(junoCfg junoconfig.Config, contractCfg ContractsConfig) Config {
	return Config{
		JunoConfig:      junoCfg,
		ContractsConfig: contractCfg,
	}
}

// Creator represents a configuration creator
func Creator(_ *cobra.Command) initcmd.WritableConfig {
	return NewConfig(junoconfig.DefaultConfig(), DefaultContractsConfig())
}

// var (
// 	// Cfg represents the configuration to be used during the execution
// 	Cfg Config
// )

// // Config defines all necessary juno configuration parameters.
// type Config struct {
// 	bytes []byte
//
// 	Contracts ContractsConfig        `yaml:"contracts"`
// 	Chain     junoconfig.ChainConfig `yaml:"chain"`
// 	Node      nodeconfig.Config      `yaml:"node"`
// 	Parser    parserconfig.Config    `yaml:"parsing"`
// 	Database  databaseconfig.Config  `yaml:"database"`
// 	Logging   loggingconfig.Config   `yaml:"logging"`
// }
//
// // NewConfig builds a new Config instance
// func NewConfig(
// 	nodeCfg nodeconfig.Config, contractCfg ContractsConfig,
// 	chainCfg junoconfig.ChainConfig, dbConfig databaseconfig.Config,
// 	parserConfig parserconfig.Config, loggingConfig loggingconfig.Config,
// ) Config {
// 	return Config{
// 		Node:      nodeCfg,
// 		Contracts: contractCfg,
// 		Chain:     chainCfg,
// 		Database:  dbConfig,
// 		Parser:    parserConfig,
// 		Logging:   loggingConfig,
// 	}
// }
//
// func DefaultConfig() Config {
// 	cfg := NewConfig(
// 		nodeconfig.DefaultConfig(), DefaultContractsConfig(),
// 		junoconfig.DefaultChainConfig(), databaseconfig.DefaultDatabaseConfig(),
// 		parserconfig.DefaultParsingConfig(), loggingconfig.DefaultLoggingConfig(),
// 	)
//
// 	bz, err := yaml.Marshal(cfg)
// 	if err != nil {
// 		panic(err)
// 	}
//
// 	cfg.bytes = bz
// 	return cfg
// }

func (c Config) GetBytes() ([]byte, error) {
	return yaml.Marshal(&c)
}

// ---------------------------------------------------------------------------------------------------------------------

// ContractsConfig specify contract addresses that want to listen
// key is contract name, value is contract address
type ContractsConfig map[string]string

func NewContractsConfig(contracts map[string]string) ContractsConfig {
	return contracts
}

func DefaultContractsConfig() ContractsConfig {
	contracts := map[string]string{
		"lst_token:":       "bbn1mh76nwv9q9qtl2ls9xudyfervq04w700juvs3s",
		"lst_staking_hub:": "bbn18x2se5n5nqrcx7f50wdn3aced4nuwl74cck8cv",
	}

	return NewContractsConfig(contracts)
}

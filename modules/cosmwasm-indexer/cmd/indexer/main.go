package main

import (
	junocmd "github.com/forbole/juno/v6/cmd"
	initcmd "github.com/forbole/juno/v6/cmd/init"
	parsetypes "github.com/forbole/juno/v6/cmd/parse/types"
	startcmd "github.com/forbole/juno/v6/cmd/start"

	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/logging"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/modules"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/types/config"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/utils"
)

func main() {
	// Set up the config
	initCfg := initcmd.NewConfig().WithConfigCreator(config.Creator)

	cdc := utils.GetCodec()

	parseCfg := parsetypes.NewConfig().
		WithDBBuilder(database.Builder).
		WithRegistrar(modules.NewModulesRegistrar(cdc)).
		WithLogger(logging.NewSlogLogger())

	cfg := junocmd.NewConfig("indexer").
		WithInitConfig(initCfg).
		WithParseConfig(parseCfg)

	// Run the commands and panic on any error
	rootCmd := junocmd.RootCmd(cfg.GetName())
	rootCmd.AddCommand(
		junocmd.VersionCmd(),
		initcmd.NewInitCmd(cfg.GetInitConfig()),
		startcmd.NewStartCmd(cfg.GetParseConfig()),
	)

	executor := junocmd.PrepareRootCmd(cfg.GetName(), rootCmd)
	err := executor.Execute()
	if err != nil {
		panic(err)
	}
}

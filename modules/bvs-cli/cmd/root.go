package cmd

import (
	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

func Cmd() *cobra.Command {
	rootCmd := &cobra.Command{
		Use:   "satlayer",
		Short: "Start the SatLayer app.",
	}

	keys := keysCmd()
	directory := directoryCmd()
	directoryEVM := directoryEVMCmd()
	delegation := delegationCmd()
	strategy := strategyCmd()
	strategyBase := strategyBaseCmd()
	reward := rewardCmd()
	chain := chainCmd()
	slash := slashCmd()
	slashEVM := slashEvmCmd()
	strategyBaseTVL := strategyBaseTVLCmd()
	strategyFactory := strategyFactoryCmd()
	bvsDriverEVM := bvsDriverEVMCmd()
	statBankEVM := stateBankEVMCmd()

	rootCmd.AddCommand(keys)
	rootCmd.AddCommand(directory)
	rootCmd.AddCommand(directoryEVM)
	rootCmd.AddCommand(delegation)
	rootCmd.AddCommand(strategy)
	rootCmd.AddCommand(strategyBase)
	rootCmd.AddCommand(reward)
	rootCmd.AddCommand(chain)
	rootCmd.AddCommand(slash)
	rootCmd.AddCommand(slashEVM)
	rootCmd.AddCommand(strategyBaseTVL)
	rootCmd.AddCommand(strategyFactory)
	rootCmd.AddCommand(bvsDriverEVM)
	rootCmd.AddCommand(statBankEVM)

	rootCmd.Version = conf.GetVersion()

	return rootCmd
}

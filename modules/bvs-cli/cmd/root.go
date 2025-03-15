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
	delegation := delegationCmd()
	strategy := strategyCmd()
	strategyBase := strategyBaseCmd()
	chain := chainCmd()
	slash := slashCmd()

	rootCmd.AddCommand(keys)
	rootCmd.AddCommand(directory)
	rootCmd.AddCommand(delegation)
	rootCmd.AddCommand(strategy)
	rootCmd.AddCommand(strategyBase)
	rootCmd.AddCommand(chain)
	rootCmd.AddCommand(slash)

	rootCmd.Version = conf.GetVersion()

	return rootCmd
}

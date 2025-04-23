package cmd

import (
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

func RootCmd() *cobra.Command {
	rootCmd := &cobra.Command{
		Use: "satlayer",
	}

	rootCmd.PersistentFlags().String("node", "https://babylon-testnet-rpc.nodes.guru", "Node uri, endpoint to the node, e.g. https://babylon-testnet-rpc.nodes.guru")
	rootCmd.PersistentFlags().String("chain-id", "bbn-test-5", "Chain id of the node, e.g. bbn-test-5")
	rootCmd.PersistentFlags().String("keyring-backend", "os", "Backend of the keyring to use, options: os, test, file")
	rootCmd.PersistentFlags().String("from", "", "From key to use for signing transactions, e.g. key-name")
	rootCmd.PersistentFlags().String("config", "", "Path to the config file, e.g. /path/to/config.yaml")

	_ = viper.BindPFlag("node", rootCmd.PersistentFlags().Lookup("node"))
	_ = viper.BindPFlag("chain-id", rootCmd.PersistentFlags().Lookup("chain-id"))
	_ = viper.BindPFlag("keyring-backend", rootCmd.PersistentFlags().Lookup("keyring-backend"))
	_ = viper.BindPFlag("from", rootCmd.PersistentFlags().Lookup("from"))
	_ = viper.BindPFlag("config", rootCmd.PersistentFlags().Lookup("config"))

	rootCmd.AddCommand(PauserCommand())
	rootCmd.AddCommand(RewardsCommand())
	return rootCmd
}

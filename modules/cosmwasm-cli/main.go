package main

import (
	"fmt"
	"os"

	"github.com/satlayer/satlayer-bvs/cosmwasm-cli/cmd"
	"github.com/satlayer/satlayer-bvs/cosmwasm-cli/sdk"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

func main() {
	rootCmd := &cobra.Command{
		Use: "satlayer",
	}

	cobra.OnInitialize(initConfig)
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

	clientCtx := sdk.NewClientCtx()
	broadcastOpts := sdk.DefaultBroadcastOptions()

	rootCmd.AddCommand(cmd.PauserCommand(clientCtx, broadcastOpts))

	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

func initConfig() {
	cfgFile := viper.GetString("config")
	if cfgFile != "" {
		viper.SetConfigFile(cfgFile)
		if err := viper.ReadInConfig(); err != nil {
			fmt.Printf("Error reading config file: %s\n", err)
			os.Exit(1)
		}
	}
}

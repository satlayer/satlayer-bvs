package main

import (
	"fmt"
	"os"

	"github.com/satlayer/satlayer-bvs/cosmwasm-cli/cmd"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

func main() {
	cobra.OnInitialize(initConfig)
	rootCmd := cmd.RootCmd()
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

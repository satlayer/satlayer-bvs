package cmd

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var RootCmd = &cobra.Command{
	Use:   "bvs-rewards",
	Short: "bvs-rewards is a command line tool to manage rewards",
	Long:  `bvs-rewards is a command line tool to manage rewards`,
}

func Execute() {
	if err := RootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

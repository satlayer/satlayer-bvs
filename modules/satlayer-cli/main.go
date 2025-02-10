package main

import (
	"fmt"
	"os"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/cmd"
)

func main() {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("error:", r)
			os.Exit(1)
		}
	}()
	rootCmd := cmd.Cmd()
	// execute command
	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

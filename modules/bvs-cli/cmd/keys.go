package cmd

import (
	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/keys"
)

func keysCmd() *cobra.Command {

	subCmd := &cobra.Command{
		Use:   "keys",
		Short: "Keys related commands",
	}

	showCmd := &cobra.Command{
		Use:   "show <keyName>",
		Short: "To show the key detail",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			keys.Show(args[0])
		},
	}

	checkCmd := &cobra.Command{
		Use:   "check <keyName>",
		Short: "To check the key",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			keys.Check(args[0])
		},
	}

	subCmd.AddCommand(showCmd)
	subCmd.AddCommand(checkCmd)

	return subCmd
}

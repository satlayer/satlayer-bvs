package cmd

import (
	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/commands/keys"
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

	evmCreateAccount := &cobra.Command{
		Use:   "evm-create-account <password>",
		Short: "To create evm account",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			keys.EVMCreateAccount(args[0])
		},
	}

	evmImportKey := &cobra.Command{
		Use:   "evm-import-key <privateKey> <password>",
		Short: "To create evm account",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			keys.EVMImportKey(args[0], args[1])
		},
	}

	subCmd.AddCommand(showCmd)
	subCmd.AddCommand(checkCmd)
	subCmd.AddCommand(evmCreateAccount)
	subCmd.AddCommand(evmImportKey)

	return subCmd
}

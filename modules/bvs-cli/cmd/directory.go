package cmd

import (
	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/directory"
)

func directoryCmd() *cobra.Command {
	subCmd := &cobra.Command{
		Use:   "directory",
		Short: "Directory related commands",
	}

	statusCmd := &cobra.Command{
		Use:   "status <operator> <service>",
		Short: "To registration status of the operator to service.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			directory.Status(args[0], args[1])
		},
	}

	transferOwnerCmd := &cobra.Command{
		Use:   "transfer-owner <userKeyName> <newOwner>",
		Short: "To transfer the ownership of the directory contract.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			directory.TransferOwner(args[0], args[1])
		},
	}

	subCmd.AddCommand(statusCmd)
	subCmd.AddCommand(transferOwnerCmd)
	return subCmd
}

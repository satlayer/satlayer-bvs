package cmd

import (
	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/commands/directoryevm"
)

func directoryEVMCmd() *cobra.Command {
	subCmd := &cobra.Command{
		Use:   "directory-evm",
		Short: "Directory EVM contract related commands",
	}

	// reg-bvs
	regBVSCmd := &cobra.Command{
		Use:   "reg-bvs <userAddr> <password> <bvsHash> <bvsContract>",
		Short: "To register the bvs within the directory contract.",
		Args:  cobra.ExactArgs(4),
		Run: func(cmd *cobra.Command, args []string) {
			directoryevm.RegBVS(args[0], args[1], args[2], args[3])
		},
	}

	acceptOwnerShipCmd := &cobra.Command{
		Use:   "accept-owner <userAddr> <password> ",
		Short: "To accept the owner",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			directoryevm.AcceptOwner(args[0], args[1])
		},
	}

	renounceOwnerShipCmd := &cobra.Command{
		Use:   "renounce-owner <userAddr> <password> ",
		Short: "To renounce owner",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			directoryevm.RenounceOwner(args[0], args[1])
		},
	}

	transferOwnerCmd := &cobra.Command{
		Use:   "transfer-owner <userAddr> <password>  <new-owner>",
		Short: "To transfer the ownership of the directory contract.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			directoryevm.TransferOwner(args[0], args[1], args[2])
		},
	}

	getOwnerCmd := &cobra.Command{
		Use:   "get-owner",
		Short: "To get the owner details from the directory contract.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			directoryevm.GetOwner()
		},
	}
	getPendingOwnerCmd := &cobra.Command{
		Use:   "get-pending-owner",
		Short: "To get the pending owner details from the directory contract.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			directoryevm.GetOwner()
		},
	}

	getBVSInfoCmd := &cobra.Command{
		Use:   "get-bvsinfo <bvsHash>",
		Short: "To get the bvs info from the directory contract.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			directoryevm.GetBVSInfo(args[0])
		},
	}

	subCmd.AddCommand(regBVSCmd)
	subCmd.AddCommand(acceptOwnerShipCmd)
	subCmd.AddCommand(renounceOwnerShipCmd)
	subCmd.AddCommand(transferOwnerCmd)
	subCmd.AddCommand(getOwnerCmd)
	subCmd.AddCommand(getPendingOwnerCmd)
	subCmd.AddCommand(getBVSInfoCmd)
	return subCmd
}

package cmd

import (
	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/commands/bvsdriverevm"
)

func bvsDriverEVMCmd() *cobra.Command {
	subCmd := &cobra.Command{
		Use:   "bvs-driver-evm",
		Short: "BVSDriver EVM contract related commands",
	}

	// reg-bvs
	regBVSCmd := &cobra.Command{
		Use:   "reg-bvs <userAddr> <password> <bvsContract>",
		Short: "To register the BVS within the BVS driver contract.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			bvsdriverevm.RegBVS(args[0], args[1], args[2])
		},
	}

	acceptOwnerShipCmd := &cobra.Command{
		Use:   "accept-owner <userAddr> <password> ",
		Short: "To accept the owner",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			bvsdriverevm.AcceptOwner(args[0], args[1])
		},
	}

	renounceOwnerShipCmd := &cobra.Command{
		Use:   "renounce-owner <userAddr> <password> ",
		Short: "To renounce owner",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			bvsdriverevm.RenounceOwner(args[0], args[1])
		},
	}

	transferOwnerCmd := &cobra.Command{
		Use:   "transfer-owner <userAddr> <password>  <new-owner>",
		Short: "To transfer the ownership of the BVSDriver contract.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			bvsdriverevm.TransferOwner(args[0], args[1], args[2])
		},
	}

	getOwnerCmd := &cobra.Command{
		Use:   "get-owner",
		Short: "To get the owner details from the BVSDriver contract.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			bvsdriverevm.GetOwner()
		},
	}
	getPendingOwnerCmd := &cobra.Command{
		Use:   "get-pending-owner",
		Short: "To get the pending owner details from the BVSDriver contract.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			bvsdriverevm.GetPendingOwner()
		},
	}

	isBVSRegistered := &cobra.Command{
		Use:   "is-bvs-registered <bvsContract>",
		Short: "To check the bvs contract is registered",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			bvsdriverevm.IsBVSRegistered(args[0])
		},
	}

	subCmd.AddCommand(regBVSCmd)
	subCmd.AddCommand(acceptOwnerShipCmd)
	subCmd.AddCommand(renounceOwnerShipCmd)
	subCmd.AddCommand(transferOwnerCmd)
	subCmd.AddCommand(getOwnerCmd)
	subCmd.AddCommand(getPendingOwnerCmd)
	subCmd.AddCommand(isBVSRegistered)
	return subCmd
}

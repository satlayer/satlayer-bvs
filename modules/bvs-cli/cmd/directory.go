package cmd

import (
	"fmt"
	"strconv"

	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/directory"
)

func directoryCmd() *cobra.Command {
	subCmd := &cobra.Command{
		Use:   "directory",
		Short: "Directory related commands",
	}

	// reg-bvs
	regBVSCmd := &cobra.Command{
		Use:   "reg-bvs <userKeyName> <bvsAddr>",
		Short: "To register the bvs within the directory contract.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			directory.RegBVS(args[0], args[1])
		},
	}

	regOperatorCmd := &cobra.Command{
		Use:   "reg-operator <operatorKeyName>",
		Short: "To register the operator within the directory contract.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			directory.RegOperator(args[0])
		},
	}

	deregOperatorCmd := &cobra.Command{
		Use:   "dereg-operator <operatorKeyName>",
		Short: "To deregister the operator within the directory contract.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			directory.DeregOperator(args[0])
		},
	}

	updateMetadataCmd := &cobra.Command{
		Use:   "update-metadata <userKeyName> <metadata>",
		Short: "To update the metadata of the operator within the directory contract.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			directory.UpdateMetadata(args[0], args[1])
		},
	}

	cancelSaltCmd := &cobra.Command{
		Use:   "cancel-salt <userKeyName> <salt>",
		Short: "To cancel the salt within the directory contract.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			directory.CancelSalt(args[0], args[1])
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

	pauseCmd := &cobra.Command{
		Use:   "pause <userKeyName>",
		Short: "To pause the directory contract.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			directory.Pause(args[0])
		},
	}

	unpauseCmd := &cobra.Command{
		Use:   "unpause <userKeyName>",
		Short: "To unpause the directory contract.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			directory.Unpause(args[0])
		},
	}

	setPauserCmd := &cobra.Command{
		Use:   "set-pauser <userKeyName> <newPauser>",
		Short: "To set the pauser of the directory contract.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			directory.SetPauser(args[0], args[1])
		},
	}

	setUnpauserCmd := &cobra.Command{
		Use:   "set-unpauser <userKeyName> <newUnpauser>",
		Short: "To set the unpauser of the directory contract.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			directory.SetUnpauser(args[0], args[1])
		},
	}

	getOperatorCmd := &cobra.Command{
		Use:   "get-operator <operatorAddress>",
		Short: "To get the operator details from the directory contract.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			directory.GetOperator(args[0])
		},
	}

	calcDigesthashCmd := &cobra.Command{
		Use:   "calc-digesthash <UserKeyName> <salt> <expiry>",
		Short: "To calculate the digest hash for the operator.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			expiry, err := strconv.ParseUint(args[2], 10, 64)
			if err != nil {
				panic(fmt.Sprintf("expire must be an integer. Error: %s\n", err))
			}
			directory.CalcDigestHash(args[0], args[1], expiry)
		},
	}

	isSaltSpentCmd := &cobra.Command{
		Use:   "is-salt-spent <address> <salt>",
		Short: "To check if the salt is spent.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			directory.IsSaltSpent(args[0], args[1])
		},
	}

	getDelegationManagerCmd := &cobra.Command{
		Use:   "get-delegation-manager",
		Short: "To get the delegation manager details from the directory contract.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			directory.GetDelegationManager()
		},
	}

	getOwnerCmd := &cobra.Command{
		Use:   "get-owner",
		Short: "To get the owner details from the directory contract.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			directory.GetOwner()
		},
	}

	getOperatorRegTypehashCmd := &cobra.Command{
		Use:   "get-operator-reg-typehash",
		Short: "To get the operator registration typehash from the directory contract.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			directory.GetOperatorBVSRegistrationTypeHash()
		},
	}

	getDomainTypehashCmd := &cobra.Command{
		Use:   "get-domain-typehash",
		Short: "To get the domain typehash from the directory contract.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			directory.GetDomainTypeHash()
		},
	}

	getDomainNameCmd := &cobra.Command{
		Use:   "get-domain-name",
		Short: "To get the domain name from the directory contract.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			directory.GetDomainName()
		},
	}

	getBVSInfoCmd := &cobra.Command{
		Use:   "get-bvsinfo <bvsHash>",
		Short: "To get the bvs info from the directory contract.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			directory.GetBVSInfo(args[0])
		},
	}

	subCmd.AddCommand(regBVSCmd)
	subCmd.AddCommand(regOperatorCmd)
	subCmd.AddCommand(deregOperatorCmd)
	subCmd.AddCommand(updateMetadataCmd)
	subCmd.AddCommand(cancelSaltCmd)
	subCmd.AddCommand(transferOwnerCmd)
	subCmd.AddCommand(pauseCmd)
	subCmd.AddCommand(unpauseCmd)
	subCmd.AddCommand(setPauserCmd)
	subCmd.AddCommand(setUnpauserCmd)
	subCmd.AddCommand(getOperatorCmd)
	subCmd.AddCommand(calcDigesthashCmd)
	subCmd.AddCommand(isSaltSpentCmd)
	subCmd.AddCommand(getDelegationManagerCmd)
	subCmd.AddCommand(getOwnerCmd)
	subCmd.AddCommand(getOperatorRegTypehashCmd)
	subCmd.AddCommand(getDomainTypehashCmd)
	subCmd.AddCommand(getDomainNameCmd)
	subCmd.AddCommand(getBVSInfoCmd)
	return subCmd
}

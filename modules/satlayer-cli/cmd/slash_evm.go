package cmd

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/commands/slashevm"
)

func slashEvmCmd() *cobra.Command {
	subCmd := &cobra.Command{
		Use:   "slash-evm",
		Short: "Slash related commands",
	}
	setSlasherCmd := &cobra.Command{
		Use:   "set-slasher <userAddr> <password> <slasher>",
		Short: "To set the slasher.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			slashevm.SetSlasher(args[0], args[1], args[2])
		},
	}
	setMinimalSlashSignatureCmd := &cobra.Command{
		Use:   "set-minimal-slash-signature <userAddr> <password> <minimalSignature>",
		Short: "To set the minimal slash signature.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			signature, err := strconv.ParseInt(args[1], 10, 64)
			if err != nil {
				fmt.Printf("Error: %v\n", err)
				return
			}
			slashevm.SetMinimalSlashSignature(args[0], args[1], signature)
		},
	}
	setSlasherValidatorCmd := &cobra.Command{
		Use:   "set-slasher-validator <userAddr> <password> <validators> <values>",
		Short: "To set slasher validators.",
		Args:  cobra.ExactArgs(4),
		Run: func(cmd *cobra.Command, args []string) {
			validators := strings.Split(args[2], ",")
			values := strings.Split(args[3], ",")
			var boolValues []bool
			for _, v := range values {
				b, err := strconv.ParseBool(v)
				if err != nil {
					fmt.Printf("Error: %v\n", err)
					return
				}
				boolValues = append(boolValues, b)
			}
			slashevm.SetSlasherValidator(args[0], args[1], validators, boolValues)
		},
	}
	pauseCmd := &cobra.Command{
		Use:   "pause <userAddr> <password>",
		Short: "To pause the slash.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			slashevm.Pause(args[0], args[1])
		},
	}
	unpauseCmd := &cobra.Command{
		Use:   "unpause <userAddr> <password>",
		Short: "To unpause the slash.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			slashevm.Unpause(args[0], args[1])
		},
	}
	transferOwnershipCmd := &cobra.Command{
		Use:   "transfer-ownership <userAddr> <password> <newOwner>",
		Short: "To transfer ownership.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			slashevm.TransferOwnership(args[0], args[1], args[2])
		},
	}
	submitSlashRequestCmd := &cobra.Command{
		Use:   "submit-slash-request <userAddr> <password> <slasher> <operator> <share> <slashSignature> <slashValidators> <reason> <startTime> <endTime> <status> <validatorsPublicKeys>",
		Short: "To submit a slash request.",
		Args:  cobra.ExactArgs(12),
		Run: func(cmd *cobra.Command, args []string) {
			slashValidators := strings.Split(args[6], ",")
			validatorsPublicKeys := strings.Split(args[11], ",")
			slashSignature, _ := strconv.ParseInt(args[5], 10, 64)
			share, _ := strconv.ParseInt(args[4], 10, 64)
			startTime, _ := strconv.ParseInt(args[8], 10, 64)
			endTime, _ := strconv.ParseInt(args[9], 10, 64)
			status, _ := strconv.ParseBool(args[10])
			slashevm.SubmitSlashRequest(args[0], args[1], args[2], args[3], share, uint16(slashSignature), slashValidators, args[6], startTime, endTime, status, validatorsPublicKeys)
		},
	}
	executeSlashRequestCmd := &cobra.Command{
		Use:   "execute-slash-request <userAddr> <password> <signatures> <validatorsPublicKeys>  <slashHash>",
		Short: "To execute a slash request.",
		Args:  cobra.ExactArgs(5),
		Run: func(cmd *cobra.Command, args []string) {
			signatures := strings.Split(args[2], ",")
			validatorsPublicKeys := strings.Split(args[3], ",")
			slashevm.ExecuteSlashRequest(args[0], args[1], signatures, validatorsPublicKeys, args[4])
		},
	}
	cancelSlashRequestCmd := &cobra.Command{
		Use:   "cancel-slash-request <userAddr> <password> <slashHash>",
		Short: "To cancel a slash request.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			slashevm.CancelSlashRequest(args[0], args[1], args[2])
		},
	}

	isValidatorCmd := &cobra.Command{
		Use:   "is-validator <validator>",
		Short: "To check if an address is a validator.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			slashevm.IsValidator(args[0])
		},
	}

	isPausedCmd := &cobra.Command{
		Use:   "is-paused",
		Short: "To check if an address is a validator.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			slashevm.IsPaused()
		},
	}

	getOwnerCmd := &cobra.Command{
		Use:   "get-owner",
		Short: "Get owner address",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			slashevm.GetOwner()
		},
	}

	getPendingOwnerCmd := &cobra.Command{
		Use:   "get-pending-owner",
		Short: "Get pending owner address",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			slashevm.GetPendingOwner()
		},
	}
	getSlasherCmd := &cobra.Command{
		Use:   "get-slasher",
		Short: "Get slasher address.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			slashevm.GetSlasher()
		},
	}

	getMinimalSlashSignatureCmd := &cobra.Command{
		Use:   "get-minimal-slash-signature",
		Short: "To get the minimal slash signature value.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			slashevm.GetMinimalSlashSignature()
		},
	}

	subCmd.AddCommand(setSlasherCmd)
	subCmd.AddCommand(setMinimalSlashSignatureCmd)
	subCmd.AddCommand(setSlasherValidatorCmd)
	subCmd.AddCommand(pauseCmd)
	subCmd.AddCommand(unpauseCmd)
	subCmd.AddCommand(transferOwnershipCmd)
	subCmd.AddCommand(submitSlashRequestCmd)
	subCmd.AddCommand(executeSlashRequestCmd)
	subCmd.AddCommand(cancelSlashRequestCmd)
	subCmd.AddCommand(isValidatorCmd)
	subCmd.AddCommand(isPausedCmd)
	subCmd.AddCommand(getOwnerCmd)
	subCmd.AddCommand(getPendingOwnerCmd)
	subCmd.AddCommand(getSlasherCmd)
	subCmd.AddCommand(getMinimalSlashSignatureCmd)

	return subCmd
}

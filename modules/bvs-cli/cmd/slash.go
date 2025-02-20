package cmd

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/slash"
)

func slashCmd() *cobra.Command {
	subCmd := &cobra.Command{
		Use:   "slash",
		Short: "Slash related commands",
	}
	setSlasherCmd := &cobra.Command{
		Use:   "set-slasher <userKeyName> <slasher> <value>",
		Short: "To set the slasher.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			value, err := strconv.ParseBool(args[2])
			if err != nil {
				fmt.Printf("Error: %v\n", err)
				return
			}
			slash.SetSlasher(args[0], args[1], value)
		},
	}
	setDelegationManagerCmd := &cobra.Command{
		Use:   "set-delegation-manager <userKeyName> <delegationManager>",
		Short: "To set the delegation manager.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			slash.SetDelegationManager(args[0], args[1])
		},
	}
	setMinimalSlashSignatureCmd := &cobra.Command{
		Use:   "set-minimal-slash-signature <userKeyName> <minimalSignature>",
		Short: "To set the minimal slash signature.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			signature, err := strconv.ParseInt(args[1], 10, 64)
			if err != nil {
				fmt.Printf("Error: %v\n", err)
				return
			}
			slash.SetMinimalSlashSignature(args[0], signature)
		},
	}
	setPauserCmd := &cobra.Command{
		Use:   "set-pauser <userKeyName> <pauser>",
		Short: "To set the pauser.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			slash.SetPauser(args[0], args[1])
		},
	}
	setUnpauserCmd := &cobra.Command{
		Use:   "set-unpauser <userKeyName> <unpauser>",
		Short: "To set the unpauser.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			slash.SetUnpauser(args[0], args[1])
		},
	}
	setSlasherValidatorCmd := &cobra.Command{
		Use:   "set-slasher-validator <userKeyName> <validators> <values>",
		Short: "To set slasher validators.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			validators := strings.Split(args[1], ",")
			values := strings.Split(args[2], ",")
			var boolValues []bool
			for _, v := range values {
				b, err := strconv.ParseBool(v)
				if err != nil {
					fmt.Printf("Error: %v\n", err)
					return
				}
				boolValues = append(boolValues, b)
			}
			slash.SetSlasherValidator(args[0], validators, boolValues)
		},
	}
	pauseCmd := &cobra.Command{
		Use:   "pause <userKeyName>",
		Short: "To pause the slash.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			slash.Pause(args[0])
		},
	}
	unpauseCmd := &cobra.Command{
		Use:   "unpause <userKeyName>",
		Short: "To unpause the slash.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			slash.Unpause(args[0])
		},
	}
	transferOwnershipCmd := &cobra.Command{
		Use:   "transfer-ownership <userKeyName> <newOwner>",
		Short: "To transfer ownership.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			slash.TransferOwnership(args[0], args[1])
		},
	}
	submitSlashRequestCmd := &cobra.Command{
		Use:   "submit-slash-request <userKeyNames> <slasher> <operator> <share> <slashSignature> <slashValidators> <reason> <startTime> <endTime> <status>",
		Short: "To submit a slash request.",
		Args:  cobra.ExactArgs(10),
		Run: func(cmd *cobra.Command, args []string) {
			keyNames := strings.Split(args[0], ",")
			slashValidators := strings.Split(args[5], ",")
			slashSignature, _ := strconv.ParseInt(args[4], 10, 64)
			startTime, _ := strconv.ParseInt(args[7], 10, 64)
			endTime, _ := strconv.ParseInt(args[8], 10, 64)
			status, _ := strconv.ParseBool(args[9])
			slash.SubmitSlashRequest(keyNames, args[1], args[2], args[3], slashSignature, slashValidators, args[6], startTime, endTime, status)
		},
	}
	executeSlashRequestCmd := &cobra.Command{
		Use:   "execute-slash-request <userKeyNames> <slashHash>",
		Short: "To execute a slash request.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			keyNames := strings.Split(args[0], ",")
			slash.ExecuteSlashRequest(keyNames, args[1])
		},
	}
	cancelSlashRequestCmd := &cobra.Command{
		Use:   "cancel-slash-request <userKeyNames> <slashHash>",
		Short: "To cancel a slash request.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			slash.CancelSlashRequest(args[0], args[1])
		},
	}

	getSlashDetailsCmd := &cobra.Command{
		Use:   "get-slash-details <slashHash>",
		Short: "To get the details of a slash request.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			slash.GetSlashDetails(args[0])
		},
	}

	isValidatorCmd := &cobra.Command{
		Use:   "is-validator <validator>",
		Short: "To check if an address is a validator.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			slash.IsValidator(args[0])
		},
	}

	getMinimalSlashSignatureCmd := &cobra.Command{
		Use:   "get-minimal-slash-signature",
		Short: "To get the minimal slash signature value.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			slash.GetMinimalSlashSignature()
		},
	}

	calculateSlashHashCmd := &cobra.Command{
		Use:   "calculate-slash-hash <userKeyNames> <sender> <slasher> <operator> <share> <slashSignature> <slashValidators> <reason> <startTime> <endTime> <status>",
		Short: "To calculate the hash of a slash request.",
		Args:  cobra.ExactArgs(11),
		Run: func(cmd *cobra.Command, args []string) {
			keyNames := strings.Split(args[0], ",")
			sender := args[1]
			slasher := args[2]
			operator := args[3]
			share := args[4]
			slashSignature, _ := strconv.ParseInt(args[5], 10, 64)
			slashValidators := strings.Split(args[6], ",")
			reason := args[7]
			startTime, _ := strconv.ParseInt(args[8], 10, 64)
			endTime, _ := strconv.ParseInt(args[9], 10, 64)
			status, _ := strconv.ParseBool(args[10])

			slash.CalculateSlashHash(keyNames, sender, slasher, operator, share, slashSignature, slashValidators, reason, startTime, endTime, status)
		},
	}

	subCmd.AddCommand(setSlasherCmd)
	subCmd.AddCommand(setDelegationManagerCmd)
	subCmd.AddCommand(setMinimalSlashSignatureCmd)
	subCmd.AddCommand(setPauserCmd)
	subCmd.AddCommand(setUnpauserCmd)
	subCmd.AddCommand(setSlasherValidatorCmd)
	subCmd.AddCommand(pauseCmd)
	subCmd.AddCommand(unpauseCmd)
	subCmd.AddCommand(transferOwnershipCmd)
	subCmd.AddCommand(submitSlashRequestCmd)
	subCmd.AddCommand(executeSlashRequestCmd)
	subCmd.AddCommand(cancelSlashRequestCmd)
	subCmd.AddCommand(getSlashDetailsCmd)
	subCmd.AddCommand(isValidatorCmd)
	subCmd.AddCommand(getMinimalSlashSignatureCmd)
	subCmd.AddCommand(calculateSlashHashCmd)

	return subCmd
}

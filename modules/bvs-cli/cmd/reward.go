package cmd

import (
	"fmt"
	"strconv"

	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/reward"
)

func rewardCmd() *cobra.Command {
	subCmd := &cobra.Command{
		Use:   "reward",
		Short: "Reward related commands",
	}

	setClaimerCmd := &cobra.Command{
		Use:   "set-claimer <userKeyName> <claimerAddress>",
		Short: "To set the claimer.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			reward.SetClaimer(args[0], args[1])
		},
	}
	setActivationDelayCmd := &cobra.Command{
		Use:   "set-activation-delay  <userKeyName> <activationDelay>",
		Short: "To set the activation delay.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			activationDelay, err := strconv.ParseInt(args[1], 10, 64)
			if err != nil {
				fmt.Printf("Cmd Args: %s\n", args)
			}
			reward.SetActivationDelay(args[0], activationDelay)
		},
	}
	setGlobalOperatorCommissionCmd := &cobra.Command{
		Use:   "set-global-operator-commission <userKeyName> <newCommissionBips>",
		Short: "To set the global operator commission.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			newCommissionBips, err := strconv.ParseInt(args[1], 10, 64)
			if err != nil {
				fmt.Printf("Cmd Args: %s\n", args)
			}
			reward.SetGlobalOperatorCommission(args[0], newCommissionBips)
		},
	}
	setRewardsUpdaterCmd := &cobra.Command{
		Use:   "set-rewards-updater <userKeyName> <RewardsUpdaterAddress>",
		Short: "To set the rewards updater.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			reward.SetRewardUpdater(args[0], args[1])
		},
	}
	transferOwnerCmd := &cobra.Command{
		Use:   "transfer-owner <userKeyName> <NewOwnerAddress>",
		Short: "To transfer the ownership.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			reward.TransferOwner(args[0], args[1])
		},
	}

	getDistributionRootLengthCmd := &cobra.Command{
		Use:   "get-distribution-root-length",
		Short: "To get the distribution root length.",
		Args:  cobra.ExactArgs(0),
		Run: func(cmd *cobra.Command, args []string) {
			reward.GetDistributionRootLength()
		},
	}
	getCurrentDistributionRootCmd := &cobra.Command{
		Use:   "get-current-distribution-root",
		Short: "To get the current distribution root.",
		Args:  cobra.ExactArgs(0),
		Run: func(cmd *cobra.Command, args []string) {
			reward.GetCurrentDistributionRoot()
		},
	}
	getDistributionRootAtIndexCmd := &cobra.Command{
		Use:   "get-distribution-root-at-index <index>",
		Short: "To get the distribution root at index.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			reward.GetDistributionRootAtIndex(args[0])
		},
	}
	getCurrentClaimableDistributionRootCmd := &cobra.Command{
		Use:   "get-current-claimable-distribution-root",
		Short: "To get the current claimable distribution root.",
		Args:  cobra.ExactArgs(0),
		Run: func(cmd *cobra.Command, args []string) {
			reward.GetCurrentClaimableDistributionRoot()
		},
	}
	getRootIndexFromHashCmd := &cobra.Command{
		Use:   "get-root-index-from-hash <hash>",
		Short: "To get the root index from hash.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			reward.GetRootIndexFromHash(args[0])
		},
	}

	subCmd.AddCommand(setClaimerCmd)
	subCmd.AddCommand(setActivationDelayCmd)
	subCmd.AddCommand(setGlobalOperatorCommissionCmd)
	subCmd.AddCommand(setRewardsUpdaterCmd)
	subCmd.AddCommand(transferOwnerCmd)
	subCmd.AddCommand(getDistributionRootLengthCmd)
	subCmd.AddCommand(getCurrentDistributionRootCmd)
	subCmd.AddCommand(getDistributionRootAtIndexCmd)
	subCmd.AddCommand(getCurrentClaimableDistributionRootCmd)
	subCmd.AddCommand(getRootIndexFromHashCmd)

	return subCmd
}

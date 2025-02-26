package cmd

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/delegation"
)

func delegationCmd() *cobra.Command {
	subCmd := &cobra.Command{
		Use:   "delegation",
		Short: "Delegation related commands",
	}

	regOperatorCmd := &cobra.Command{
		Use:   "reg-operator <operatorKeyName>",
		Short: "To register the operator within the delegation contract.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.RegOperator(args[0])
		},
	}
	updateOperatorDetailsCmd := &cobra.Command{
		Use:   "update-operator-details <userKeyName> <deprecatedEarningsReceiver> <stakerOptOutWindowBlocks>",
		Short: "To update the operator details within the delegation contract.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			windowBlocks, err := strconv.ParseInt(args[2], 10, 64)
			if err != nil {
				panic(fmt.Sprintf("expire must be an integer. Error: %s\n", err))
			}
			delegation.UpdateOperatorDetails(args[0], args[1], windowBlocks)
		},
	}

	updateOperatorMetadatauriCmd := &cobra.Command{
		Use:   "update-operator-metadatauri <userKeyName> <uri>",
		Short: "To update the operator metadata uri within the delegation contract.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.UpdateOperatorMetadataURI(args[0], args[1])
		},
	}

	delegateToCmd := &cobra.Command{
		Use:   "delegate-to <stakerKeyName> <operatorAddress>",
		Short: "To delegate to the operator.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.DelegateTo(args[0], args[1])
		},
	}

	undelegateCmd := &cobra.Command{
		Use:   "undelegate <stakerKeyName> <operatorAddress>",
		Short: "To undelegate the operator.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.Undelegate(args[0], args[1])
		},
	}

	setMinWithdrawDelayBlocksCmd := &cobra.Command{
		Use:   "set-minwithdraw-delayblocks <userKeyName> <minWithdrawDelayBlocks>",
		Short: "To set the min withdraw delay blocks.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			blocks, err := strconv.ParseInt(args[1], 10, 64)
			if err != nil {
				panic(fmt.Sprintf("minWithdrawDelayBlocks must be an integer. Error: %s\n", err))
			}
			delegation.SetMinWithdrawDelayBlocks(args[0], blocks)
		},
	}

	setStrategywithdrawDelayblocksCmd := &cobra.Command{
		Use:   "set-strategywithdraw-delayblocks <userKeyName> <StrategyAddress> <blockNumbers>",
		Short: "To set the strategy withdraw delay blocks.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			strategies, blocks := strings.Split(args[1], ","), strings.Split(args[2], ",")
			newBlocks := make([]int64, len(blocks))
			var err error
			for i := 0; i < len(blocks); i++ {
				newBlocks[i], err = strconv.ParseInt(blocks[i], 10, 64)
				if err != nil {
					panic(fmt.Sprintf("blockNumbers must be an integer. Error: %s\n", err))
				}
			}
			delegation.SetStrategyWithdrawDelayBlocks(args[0], strategies, newBlocks)
		},
	}
	transferOwnerCmd := &cobra.Command{
		Use:   "transfer-owner <userKeyName> <newOwner>",
		Short: "To transfer the owner.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.TransferOwnership(args[0], args[1])
		},
	}

	pauseCmd := &cobra.Command{
		Use:   "pause <userKeyName>",
		Short: "To pause the contract.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.Pause(args[0])
		},
	}

	unpauseCmd := &cobra.Command{
		Use:   "unpause <userKeyName>",
		Short: "To unpause the contract.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.Unpause(args[0])
		},
	}

	setPauserCmd := &cobra.Command{
		Use:   "set-pauser <userKeyName> <pauser>",
		Short: "To set the pauser.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.SetPauser(args[0], args[1])
		},
	}

	setUnpauserCmd := &cobra.Command{
		Use:   "set-unpauser <userKeyName> <unpauser>",
		Short: "To set the unpauser.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.SetUnpauser(args[0], args[1])
		},
	}

	isDelegatedCmd := &cobra.Command{
		Use:   "is-delegated <stakerAddress>",
		Short: "To check if the operator is delegated.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.IsDelegated(args[0])
		},
	}

	isOperatorCmd := &cobra.Command{
		Use:   "is-operator <operatorAddress>",
		Short: "To check if the operator is operator.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.IsOperator(args[0])
		},
	}

	getOperatorDetailsCmd := &cobra.Command{
		Use:   "get-operator-details <operatorAddress>",
		Short: "To get the operator details.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.GetOperatorDetails(args[0])
		},
	}

	getOperatorStakersCmd := &cobra.Command{
		Use:   "get-operator-stakers <operatorAddress>",
		Short: "To get the operator stakers.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.GetOperatorStakers(args[0])
		},
	}

	getStakerOptoutwindowblocksCmd := &cobra.Command{
		Use:   "get-staker-optoutwindowblocks <operatorAddress>",
		Short: "To get the staker optoutwindowblocks.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.GetStakerOptOutWindowBlocks(args[0])
		},
	}

	getOperatorSharesCmd := &cobra.Command{
		Use:   "get-operator-shares <operatorAddress> <strategyAddress>",
		Short: "To get the operator shares.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategies := strings.Split(args[1], ",")
			delegation.GetOperatorShares(args[0], strategies)
		},
	}

	getDelegatableSharesCmd := &cobra.Command{
		Use:   "get-delegatable-shares <operatorAddress>",
		Short: "To get the delegatable shares.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.GetDelegatableShares(args[0])
		},
	}

	getWithdrawDelayCmd := &cobra.Command{
		Use:   "get-withdraw-delay <strategyAddress>",
		Short: "To get the withdraw delay.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			strategies := strings.Split(args[0], ",")
			delegation.GetWithdrawDelay(strategies)
		},
	}

	calcWithdrawRootCmd := &cobra.Command{
		Use:   "calc-withdraw-root <stakerAddress> <delegatedToAddress> <withdrawerAddress> <nonce> <startBlock> <strategies> <shares>",
		Short: "To get the withdraw root.",
		Args:  cobra.ExactArgs(7),
		Run: func(cmd *cobra.Command, args []string) {
			startBlock, err := strconv.ParseInt(args[4], 10, 64)
			if err != nil {
				panic(fmt.Sprintf("startBlock must be an integer. Error: %s\n", err))
			}
			strategies := strings.Split(args[5], ",")
			shares := strings.Split(args[6], ",")
			delegation.CalcWithdrawRoot(args[0], args[1], args[2], args[3], startBlock, strategies, shares)

		},
	}

	getCumulativeWithdrawQueuednoncesCmd := &cobra.Command{
		Use:   "get-cumulative-withdraw-queuednonces <stakerAddress>",
		Short: "To get the cumulative withdraw queuednonces.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			delegation.GetCumulativeWithdrawQueueNonce(args[0])
		},
	}
	subCmd.AddCommand(regOperatorCmd)
	subCmd.AddCommand(updateOperatorDetailsCmd)
	subCmd.AddCommand(updateOperatorMetadatauriCmd)
	subCmd.AddCommand(delegateToCmd)
	subCmd.AddCommand(undelegateCmd)
	subCmd.AddCommand(setMinWithdrawDelayBlocksCmd)
	subCmd.AddCommand(setStrategywithdrawDelayblocksCmd)
	subCmd.AddCommand(transferOwnerCmd)
	subCmd.AddCommand(pauseCmd)
	subCmd.AddCommand(unpauseCmd)
	subCmd.AddCommand(setPauserCmd)
	subCmd.AddCommand(setUnpauserCmd)
	subCmd.AddCommand(getOperatorDetailsCmd)
	subCmd.AddCommand(getStakerOptoutwindowblocksCmd)
	subCmd.AddCommand(getOperatorSharesCmd)
	subCmd.AddCommand(getOperatorStakersCmd)
	subCmd.AddCommand(getDelegatableSharesCmd)
	subCmd.AddCommand(getWithdrawDelayCmd)
	subCmd.AddCommand(calcWithdrawRootCmd)
	subCmd.AddCommand(getCumulativeWithdrawQueuednoncesCmd)
	subCmd.AddCommand(isDelegatedCmd)
	subCmd.AddCommand(isOperatorCmd)

	return subCmd
}

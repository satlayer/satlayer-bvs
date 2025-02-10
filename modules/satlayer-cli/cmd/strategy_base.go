package cmd

import (
	"fmt"
	"strconv"

	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/commands/strategybase"
)

func strategyBaseCmd() *cobra.Command {
	subCmd := &cobra.Command{
		Use:   "strategy-base",
		Short: "Strategy base related commands",
	}
	depositCmd := &cobra.Command{
		Use:   "deposit <userKeyName> <amount>",
		Short: "To deposit.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			amount, err := strconv.ParseUint(args[1], 10, 64)
			if err != nil {
				panic(fmt.Sprintf("amount must be an integer. Error: %s\n", err))
			}
			strategybase.Deposit(args[0], amount)
		},
	}
	withdrawCmd := &cobra.Command{
		Use:   "withdraw <userKeyName> <recipient> <amount>",
		Short: "To withdraw.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			amount, err := strconv.ParseUint(args[2], 10, 64)
			if err != nil {
				panic(fmt.Sprintf("amount must be an integer. Error: %s\n", err))
			}
			strategybase.Withdraw(args[0], args[1], amount)
		},
	}
	getSharesCmd := &cobra.Command{
		Use:   "get-shares <stakerAddress> <strategyAddress>",
		Short: "To get shares.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategybase.GetShares(args[0], args[1])
		},
	}
	sharesUnderlyingviewCmd := &cobra.Command{
		Use:   "shares-underlyingview <shareAmount>",
		Short: "To get shares underlying view.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			amount, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				panic(fmt.Sprintf("amount must be an integer. Error: %s\n", err))
			}
			strategybase.SharesUnderlyingView(amount)
		},
	}
	underlyingShareviewCmd := &cobra.Command{
		Use:   "underlying-shareview <amount>",
		Short: "To get underlying share view.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			amount, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				panic(fmt.Sprintf("amount must be an integer. Error: %s\n", err))
			}
			strategybase.UnderlyingShareView(amount)
		},
	}
	underlyingViewCmd := &cobra.Command{
		Use:   "underlying-view <userAddress>",
		Short: "To get underlying view.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			strategybase.UnderlyingView(args[0])
		},
	}
	underlyingTokenCmd := &cobra.Command{
		Use:   "underlying-token",
		Short: "To get underlying token.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			strategybase.UnderlyingToken()
		},
	}
	pauseCmd := &cobra.Command{
		Use:   "pause <userKeyName>",
		Short: "To pause.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			strategybase.Pause(args[0])
		},
	}
	unpauseCmd := &cobra.Command{
		Use:   "unpause <userKeyName>",
		Short: "To unpause.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			strategybase.Unpause(args[0])
		},
	}
	setPauserCmd := &cobra.Command{
		Use:   "set-pauser <userKeyName> <pauserAddress>",
		Short: "To set the pauser.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategybase.SetPauser(args[0], args[1])
		},
	}
	setUnpauserCmd := &cobra.Command{
		Use:   "set-unpauser <userKeyName> <unpauserAddress>",
		Short: "To set the unpauser.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategybase.SetUnpauser(args[0], args[1])
		},
	}
	transferOwnerCmd := &cobra.Command{
		Use:   "transfer-owner <userKeyName> <newOwner>",
		Short: "To transfer the owner.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategybase.TransferOwnership(args[0], args[1])
		},
	}

	subCmd.AddCommand(depositCmd)
	subCmd.AddCommand(withdrawCmd)
	subCmd.AddCommand(getSharesCmd)
	subCmd.AddCommand(sharesUnderlyingviewCmd)
	subCmd.AddCommand(underlyingShareviewCmd)
	subCmd.AddCommand(underlyingViewCmd)
	subCmd.AddCommand(underlyingTokenCmd)
	subCmd.AddCommand(pauseCmd)
	subCmd.AddCommand(unpauseCmd)
	subCmd.AddCommand(setPauserCmd)
	subCmd.AddCommand(setUnpauserCmd)
	subCmd.AddCommand(transferOwnerCmd)

	return subCmd
}

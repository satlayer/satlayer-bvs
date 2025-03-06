package cmd

import (
	"fmt"
	"strconv"

	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/strategy"
)

func strategyCmd() *cobra.Command {
	subCmd := &cobra.Command{
		Use:   "strategy",
		Short: "Strategy related commands",
	}
	transferOwnerCmd := &cobra.Command{
		Use:   "transfer-owner <userKeyName> <newOwner>",
		Short: "To transfer the owner.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategy.TransferOwner(args[0], args[1])
		},
	}

	updateStrategyCmd := &cobra.Command{
		Use:   "update-strategy <userKeyName> <strategyAddress> <whitelisted>",
		Short: "To add the strategy to whitelist.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			value, err := strconv.ParseBool(args[2])
			if err != nil {
				fmt.Printf("Error: %v\n", err)
			}
			strategy.UpdateStrategy(args[0], args[1], value)
		},
	}

	depositStrategyCmd := &cobra.Command{
		Use:   "deposit-strategy <userKeyName> <strategyAddress> <tokenAddress> <amount>",
		Short: "To deposit the strategy.",
		Args:  cobra.ExactArgs(4),
		Run: func(cmd *cobra.Command, args []string) {
			amount, err := strconv.ParseUint(args[3], 10, 64)
			if err != nil {
				fmt.Printf("Error: %v\n", err)
			}
			strategy.DepositStrategy(args[0], args[1], args[2], amount)
		},
	}
	removeSharesCmd := &cobra.Command{
		Use:   "remove-shares <userKeyName> <stakerAddress> <strategyAddress> <shareAmount>",
		Short: "To remove the shares.",
		Args:  cobra.ExactArgs(4),
		Run: func(cmd *cobra.Command, args []string) {
			amount, err := strconv.ParseUint(args[3], 10, 64)
			if err != nil {
				fmt.Printf("Error: %v\n", err)
			}
			strategy.RemoveShares(args[0], args[1], args[2], amount)
		},
	}
	withdrawSharesAsTokensCmd := &cobra.Command{
		Use:   "withdraw-shares-as-tokens <userKeyName> <recipient> <strategy> <shares>",
		Short: "To withdraw the shares as tokens.",
		Args:  cobra.ExactArgs(4),
		Run: func(cmd *cobra.Command, args []string) {
			strategy.WithdrawSharesAsTokens(args[0], args[1], args[2], args[3])
		},
	}
	getDepositsCmd := &cobra.Command{
		Use:   "get-deposits <stakerAddress>",
		Short: "To get the deposits.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			strategy.GetDeposits(args[0])
		},
	}
	getStakerStrategySharesCmd := &cobra.Command{
		Use:   "get-staker-strategy-shares <stakerAddress> <strategyAddress>",
		Short: "To get the staker strategy shares.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategy.GetStakerStrategyShares(args[0], args[1])
		},
	}
	getStakerStrategyListCmd := &cobra.Command{
		Use:   "get-staker-strategy-list <stakerAddress>",
		Short: "To get the staker strategy list.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			strategy.GetStakerStrategyList(args[0])
		},
	}
	isStrategyWhitelistedCmd := &cobra.Command{
		Use:   "is-strategy-whitelisted <strategyAddress>",
		Short: "To check if the strategy is whitelisted.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			strategy.IsStrategyWhitelisted(args[0])
		},
	}

	subCmd.AddCommand(transferOwnerCmd)
	subCmd.AddCommand(getStakerStrategyListCmd)
	subCmd.AddCommand(isStrategyWhitelistedCmd)
	subCmd.AddCommand(updateStrategyCmd)
	subCmd.AddCommand(depositStrategyCmd)
	subCmd.AddCommand(removeSharesCmd)
	subCmd.AddCommand(withdrawSharesAsTokensCmd)
	subCmd.AddCommand(getDepositsCmd)
	subCmd.AddCommand(getStakerStrategySharesCmd)

	return subCmd
}

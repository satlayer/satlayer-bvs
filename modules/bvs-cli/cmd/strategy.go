package cmd

import (
	"fmt"
	"strconv"
	"strings"

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

	setDelegationManagerCmd := &cobra.Command{
		Use:   "set-delegation-manager <userKeyName> <delegationManager>",
		Short: "To set the delegation manager.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategy.SetDelegationManager(args[0], args[1])
		},
	}
	setStrategyWhitelistCmd := &cobra.Command{
		Use:   "set-strategy-whitelist <userKeyName> <strategyWhitelist>",
		Short: "To set the strategy whitelist.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategy.SetStrategyWhitelist(args[0], args[1])
		},
	}
	removeStrategyFromWhitelistCmd := &cobra.Command{
		Use:   "remove-strategy-from-whitelist <userKeyName> <strategyAddress>",
		Short: "To remove the strategy from whitelist.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategies := strings.Split(args[1], ",")
			strategy.RemoveStrategyWhitelist(args[0], strategies)
		},
	}
	addStrategyToWhitelistCmd := &cobra.Command{
		Use:   "add-strategy-to-whitelist <userKeyName> <strategyAddress>",
		Short: "To add the strategy to whitelist.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategies := strings.Split(args[1], ",")
			strategy.AddStrategyWhitelist(args[0], strategies)
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
		Use:   "withdraw-shares-as-tokens <userKeyName> <recipient> <strategy> <shares> <token>",
		Short: "To withdraw the shares as tokens.",
		Args:  cobra.ExactArgs(4),
		Run: func(cmd *cobra.Command, args []string) {
			amount, err := strconv.ParseUint(args[3], 10, 64)
			if err != nil {
				fmt.Printf("Error: %v\n", err)
			}
			strategy.WithdrawSharesAsTokens(args[0], args[1], args[2], args[4], amount)
		},
	}
	addSharesCmd := &cobra.Command{
		Use:   "add-shares <userKeyName> <stakerAddress> <tokenAddress> <strategyAddress> <shareAmount>",
		Short: "To add the shares.",
		Args:  cobra.ExactArgs(5),
		Run: func(cmd *cobra.Command, args []string) {
			amount, err := strconv.ParseUint(args[4], 10, 64)
			if err != nil {
				fmt.Printf("Error: %v\n", err)
			}
			strategy.AddShares(args[0], args[1], args[2], args[3], amount)
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

	getStakerStrategyListLengthCmd := &cobra.Command{
		Use:   "get-staker-strategy-list-length <stakerAddress>",
		Short: "To get the staker strategy list length.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			strategy.GetStakerStrategyListLength(args[0])
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
	getStrategyWhitelistCmd := &cobra.Command{
		Use:   "get-strategy-whitelist",
		Short: "To get the strategy whitelist.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			strategy.GetStrategyWhitelist()
		},
	}

	getStrategyManagerStateCmd := &cobra.Command{
		Use:   "get-strategy-manager-state",
		Short: "To get the strategy manager state.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			strategy.GetStrategyManagerState()
		},
	}

	getDelegationManagerCmd := &cobra.Command{
		Use:   "get-delegation-manager",
		Short: "To get the delegation manager.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			strategy.DelegationManager()
		},
	}

	subCmd.AddCommand(transferOwnerCmd)
	subCmd.AddCommand(setDelegationManagerCmd)
	subCmd.AddCommand(removeStrategyFromWhitelistCmd)
	subCmd.AddCommand(setStrategyWhitelistCmd)
	subCmd.AddCommand(getStakerStrategyListCmd)
	subCmd.AddCommand(isStrategyWhitelistedCmd)
	subCmd.AddCommand(getStrategyWhitelistCmd)
	subCmd.AddCommand(getStrategyManagerStateCmd)
	subCmd.AddCommand(getDelegationManagerCmd)
	subCmd.AddCommand(addStrategyToWhitelistCmd)
	subCmd.AddCommand(depositStrategyCmd)
	subCmd.AddCommand(removeSharesCmd)
	subCmd.AddCommand(withdrawSharesAsTokensCmd)
	subCmd.AddCommand(addSharesCmd)
	subCmd.AddCommand(getDepositsCmd)
	subCmd.AddCommand(getStakerStrategyListLengthCmd)
	subCmd.AddCommand(getStakerStrategySharesCmd)

	return subCmd
}

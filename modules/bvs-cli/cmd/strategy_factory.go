package cmd

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/strategyfactory"
)

func strategyFactoryCmd() *cobra.Command {
	subCmd := &cobra.Command{
		Use:   "strategy-factory",
		Short: "Strategy factory related commands",
	}
	createStrategyCmd := &cobra.Command{
		Use:   "create-strategy <userKeyName> <token> <pauser> <unpauser>",
		Short: "To create a new strategy.",
		Args:  cobra.ExactArgs(4),
		Run: func(cmd *cobra.Command, args []string) {
			strategyfactory.CreateStrategy(args[0], args[1], args[2], args[3])
		},
	}
	updateConfigCmd := &cobra.Command{
		Use:   "update-config <userKeyName> <new-owner> <strategy-code-id>",
		Short: "To update the config.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			value, err := strconv.ParseInt(args[2], 10, 64)
			if err != nil {
				fmt.Println("Error: Unable to convert input to uint64")
				return
			}
			strategyfactory.UpdateConfig(args[0], args[1], value)
		},
	}
	pauseCmd := &cobra.Command{
		Use:   "pause <userKeyName>",
		Short: "To pause.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			strategyfactory.Pause(args[0])
		},
	}
	unpauseCmd := &cobra.Command{
		Use:   "unpause <userKeyName>",
		Short: "To unpause.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			strategyfactory.Unpause(args[0])
		},
	}
	setPauserCmd := &cobra.Command{
		Use:   "set-pauser <userKeyName> <pauserAddress>",
		Short: "To set the pauser.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategyfactory.SetPauser(args[0], args[1])
		},
	}
	setUnpauserCmd := &cobra.Command{
		Use:   "set-unpauser <userKeyName> <unpauserAddress>",
		Short: "To set the unpauser.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategyfactory.SetUnpauser(args[0], args[1])
		},
	}
	transferOwnerCmd := &cobra.Command{
		Use:   "transfer-owner <userKeyName> <newOwner>",
		Short: "To transfer the owner.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategyfactory.TransferOwnership(args[0], args[1])
		},
	}
	blacklistTokensCmd := &cobra.Command{
		Use:   "blacklist-tokens <userKeyName> <tokens>",
		Short: "To blacklist tokens.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			tokens := strings.Split(args[1], ",")
			strategyfactory.BlacklistTokens(args[0], tokens)
		},
	}
	removeStrategiesFromWhitelistCmd := &cobra.Command{
		Use:   "remove-strategies-from-whitelist <userKeyName> <strategies>",
		Short: "To remove strategies from whitelist.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategies := strings.Split(args[1], ",")
			strategyfactory.RemoveStrategiesFromWhitelist(args[0], strategies)
		},
	}
	setThirdPartyTransfersForbiddenCmd := &cobra.Command{
		Use:   "set-third-party-transfers-forbidden <userKeyName> <strategy> <value>",
		Short: "To set the third party transfers forbidden.",
		Args:  cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			forbidden, err := strconv.ParseBool(args[2])
			if err != nil {
				fmt.Println("Error: Unable to convert input to bool")
				return
			}
			strategyfactory.SetThirdPartyTransfersForbidden(args[0], args[1], forbidden)
		},
	}
	whitelistStrategiesCmd := &cobra.Command{
		Use:   "whitelist-strategies <userKeyName> <strategies> <forbiddenValues>",
		Short: "To whitelist strategies.",
		Args:  cobra.MinimumNArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			strategies := strings.Split(args[1], ",")
			var forbiddenValues []bool
			values := strings.Split(args[2], ",")
			for i := 0; i < len(values); i++ {
				value, err := strconv.ParseBool(values[i])
				if err != nil {
					fmt.Printf("Error: forbiddenValues[%d] must be 'true' or 'false'\n", i)
					return
				}
				forbiddenValues = append(forbiddenValues, value)
			}
			if len(strategies) != len(forbiddenValues) {
				fmt.Println("Error: Each strategy must have a corresponding value.")
				return
			}

			strategyfactory.WhitelistStrategies(args[0], strategies, forbiddenValues)
		},
	}
	setStrategyManagerCmd := &cobra.Command{
		Use:   "set-strategy-manager <userKeyName> <newManager>",
		Short: "To set the strategy manager.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			strategyfactory.SetStrategyManager(args[0], args[1])
		},
	}
	getStrategyManagerCmd := &cobra.Command{
		Use:   "get-strategy-manager",
		Short: "To get the strategy manager.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			token := args[0]
			if token == "" {
				fmt.Println("Error: token cannot be empty")
				return
			}
			strategyfactory.GetStrategy(token)
		},
	}
	isTokenBlacklistedCmd := &cobra.Command{
		Use:   "is-token-blacklisted <token>",
		Short: "To check if a token is blacklisted.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			strategyfactory.IsTokenBlacklisted(args[0])
		},
	}

	subCmd.AddCommand(createStrategyCmd)
	subCmd.AddCommand(updateConfigCmd)
	subCmd.AddCommand(pauseCmd)
	subCmd.AddCommand(unpauseCmd)
	subCmd.AddCommand(setPauserCmd)
	subCmd.AddCommand(setUnpauserCmd)
	subCmd.AddCommand(transferOwnerCmd)
	subCmd.AddCommand(blacklistTokensCmd)
	subCmd.AddCommand(removeStrategiesFromWhitelistCmd)
	subCmd.AddCommand(setThirdPartyTransfersForbiddenCmd)
	subCmd.AddCommand(whitelistStrategiesCmd)
	subCmd.AddCommand(setStrategyManagerCmd)
	subCmd.AddCommand(getStrategyManagerCmd)
	subCmd.AddCommand(isTokenBlacklistedCmd)

	return subCmd
}

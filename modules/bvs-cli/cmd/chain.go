package cmd

import (
	"fmt"
	"strconv"

	"github.com/spf13/cobra"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/chain"
)

func chainCmd() *cobra.Command {

	subCmd := &cobra.Command{
		Use:   "chain",
		Short: "Chain related commands",
	}
	queryNodeCmd := &cobra.Command{
		Use:   "get-node",
		Short: "To query the node status info.",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			chain.QueryNode()
		},
	}
	queryTxnCmd := &cobra.Command{
		Use:   "get-txn <txnHash>",
		Short: "To query the transaction.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			chain.QueryTxn(args[0])
		},
	}
	queryAccountCmd := &cobra.Command{
		Use:   "get-account <accountAddress>",
		Short: "To query the account.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			chain.QueryAccount(args[0])
		},
	}
	increaseTokenAllowance := &cobra.Command{
		Use:   "increase-token-allowance <userKeyName> <tokenAddress> <spenderAddress> <amount>",
		Short: "To increase the token allowance.",
		Args:  cobra.ExactArgs(4),
		Run: func(cmd *cobra.Command, args []string) {
			amount, err := strconv.ParseUint(args[3], 10, 64)
			if err != nil {
				panic(fmt.Sprintf("amount must be an integer. Error: %s\n", err))
			}
			chain.IncreaseTokenAllowance(args[0], args[1], args[2], amount)
		},
	}

	subCmd.AddCommand(queryNodeCmd)
	subCmd.AddCommand(queryTxnCmd)
	subCmd.AddCommand(queryAccountCmd)
	subCmd.AddCommand(increaseTokenAllowance)

	return subCmd
}

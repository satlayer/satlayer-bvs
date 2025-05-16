package cmd

import (
	"context"
	"fmt"

	"github.com/spf13/cobra"

	cosmwasmapi "github.com/satlayer/satlayer-bvs/cosmwasm-api"
	"github.com/satlayer/satlayer-bvs/cosmwasm-cli/sdk"
	"github.com/satlayer/satlayer-bvs/cosmwasm-schema/pauser"
)

func PauserCommand() *cobra.Command {
	command := &cobra.Command{
		Use: "pauser",
	}

	// TODO(fuxingloh): need a better way to set default value based on chain-id
	command.PersistentFlags().String("contract", "", "Contract address of the pauser contract")

	command.AddCommand(pauserQuery())
	command.AddCommand(pauserExecute())
	return command
}

func pauserQuery() *cobra.Command {
	command := &cobra.Command{
		Use: "query",
	}

	command.AddCommand(&cobra.Command{
		Use:   "is_paused <contract> <method>",
		Short: "To query if the contract is paused.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			queryMsg := pauser.QueryMsg{
				IsPaused: &pauser.IsPaused{
					C: args[0],
					M: args[1],
				},
			}

			clientCtx := sdk.NewClientCtx()
			response, err := cosmwasmapi.Query[pauser.IsPausedResponse](
				clientCtx,
				context.Background(),
				cmd.Flag("contract").Value.String(),
				queryMsg,
			)
			if err != nil {
				panic(err)
			}

			paused := "unknown"
			switch response {
			case pauser.IsPausedResponse(0):
				paused = "false"
				break
			case pauser.IsPausedResponse(1):
				paused = "true"
				break
			}

			fmt.Printf("Contract: %s\nMethod: %s\nIs Paused: %s\n", args[0], args[1], paused)
		},
	})
	return command
}

func pauserExecute() *cobra.Command {
	command := &cobra.Command{
		Use: "execute",
	}

	command.AddCommand(&cobra.Command{
		Use:   "pause",
		Short: "To pause contract's methods in the ecosystem.",
		Args:  cobra.ExactArgs(2),
		Run: func(cmd *cobra.Command, args []string) {
			executeMsg := pauser.ExecuteMsg{
				Pause: &pauser.Pause{
					Contract: args[0],
					Method:   args[1],
				},
			}

			clientCtx := sdk.WithKeyring(sdk.NewClientCtx())

			opts := sdk.DefaultBroadcastOptions().
				WithContractAddr(cmd.Flag("contract").Value.String()).
				WithExecuteMsg(executeMsg)

			response, err := cosmwasmapi.Execute(
				clientCtx,
				context.Background(),
				clientCtx.GetFromAddress().String(),
				opts,
			)

			if err != nil {
				panic(err)
			}

			fmt.Printf("Transaction hash: %s\n", response.Hash.String())
		},
	})

	command.AddCommand(&cobra.Command{
		Use:   "global-pause",
		Short: "To pause all contracts in the ecosystem.",
		Args:  cobra.ExactArgs(0),
		Run: func(cmd *cobra.Command, args []string) {
			executeMsg := pauser.ExecuteMsg{
				PauseGlobal: &pauser.PauseGlobal{},
			}

			clientCtx := sdk.WithKeyring(sdk.NewClientCtx())

			opts := sdk.DefaultBroadcastOptions().
				WithContractAddr(cmd.Flag("contract").Value.String()).
				WithExecuteMsg(executeMsg)

			response, err := cosmwasmapi.Execute(
				clientCtx,
				context.Background(),
				clientCtx.GetFromAddress().String(),
				opts,
			)

			if err != nil {
				panic(err)
			}

			fmt.Printf("Transaction hash: %s\n", response.Hash.String())
		},
	})

	return command
}

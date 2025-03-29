package cmd

import (
	"context"
	"fmt"

	"github.com/satlayer/satlayer-bvs/cosmwasm-cli/sdk"
	"github.com/spf13/viper"

	"github.com/cosmos/cosmos-sdk/client"
	cosmwasmapi "github.com/satlayer/satlayer-bvs/cosmwasm-api"
	"github.com/satlayer/satlayer-bvs/cosmwasm-schema/pauser"
	"github.com/spf13/cobra"
)

func PauserCommand(clientCtx client.Context, broadcastOptions cosmwasmapi.BroadcastOptions) *cobra.Command {
	command := &cobra.Command{
		Use: "pauser",
	}

	command.AddCommand(pauserExecute(clientCtx, broadcastOptions))
	command.AddCommand(pauserQuery(clientCtx))
	return command
}

func pauserExecute(clientCtx client.Context, broadcastOptions cosmwasmapi.BroadcastOptions) *cobra.Command {
	command := &cobra.Command{
		Use: "execute",
	}

	command.AddCommand(&cobra.Command{
		Use:   "pause",
		Short: "To pause all contracts in the ecosystem.",
		Args:  cobra.ExactArgs(0),
		Run: func(cmd *cobra.Command, args []string) {
			executeMsg := pauser.ExecuteMsg{
				Pause: &pauser.Pause{},
			}

			clientCtx := sdk.WithKeyring(clientCtx)

			opts := broadcastOptions.
				WithContractAddr(viper.GetString("contracts.pauser")).
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

func pauserQuery(clientCtx client.Context) *cobra.Command {
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

			response, err := cosmwasmapi.Query[pauser.IsPausedResponse](clientCtx, context.Background(),
				viper.GetString("contracts.pauser"),
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

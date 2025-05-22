import { ExecuteResult, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Coin } from "@cosmjs/proto-signing";

import { Vote, ExecuteMsg as GovernanceExecuteMsg, Voter } from "@examples/governance-contract/governance-contract";

interface CommitteeConfig {
  client: SigningCosmWasmClient;
  vault: string;
  registry: string;
  router: string;
  service: string;
  rewards: string;
  committee: Voter[];
}

export class Committee {
  readonly client: SigningCosmWasmClient;
  readonly vault: string;
  readonly registry: string;
  readonly router: string;
  readonly operator: string;
  readonly service: string;
  readonly rewards: string;
  readonly committee: Voter[];

  constructor({ client, vault, registry, router, service, rewards, committee }: CommitteeConfig) {
    this.client = client;
    this.vault = vault;
    this.registry = registry;
    this.router = router;
    this.service = service;
    this.rewards = rewards;
    this.committee = committee;
  }

  async propose(
    action_msg: any,
    title: string,
    description: string,
    target_contract: string,
    funds: Coin[] = [],
  ): Promise<ExecuteResult> {
    let proposal: GovernanceExecuteMsg = {
      base: {
        propose: {
          title,
          description,
          msgs: [
            {
              wasm: {
                execute: {
                  contract_addr: target_contract,
                  msg: Buffer.from(JSON.stringify(action_msg)).toString("base64"),
                  funds,
                },
              },
            },
          ],
        },
      },
    };

    return await this.client.execute(this.committee[0].addr, this.service, proposal, "auto");
  }

  async all_vote_yes(proposal_id: number): Promise<void> {
    // vote on the proposal
    // skip the first member as they are the proposer
    for (let i = 1; i < this.committee.length; i++) {
      let vote: GovernanceExecuteMsg = {
        base: {
          vote: {
            proposal_id,
            vote: "yes" as Vote,
          },
        },
      };
      await this.client.execute(this.committee[i].addr, this.service, vote, "auto");
    }
  }

  async execute_proposal(proposal_id: number): Promise<ExecuteResult> {
    let msg: GovernanceExecuteMsg = {
      base: {
        execute: {
          proposal_id,
        },
      },
    };
    return await this.client.execute(this.committee[0].addr, this.service, msg, "auto");
  }
}

import { quicktype, InputData, JSONSchemaInput, FetchingJSONSchemaStore } from "quicktype-core";

import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

/**
 * @param schema {any}
 * @returns {Promise<void>}
 */
async function generate(schema) {
  const schemaInput = new JSONSchemaInput(new FetchingJSONSchemaStore());
  await schemaInput.addSource({ name: "InstantiateMsg", schema: JSON.stringify(schema.instantiate) });
  await schemaInput.addSource({ name: "ExecuteMsg", schema: JSON.stringify(schema.execute) });

  if (schema.query.enum?.length !== 0) {
    await schemaInput.addSource({ name: "QueryMsg", schema: JSON.stringify(schema.query) });
  }
  const inputData = new InputData();
  inputData.addInput(schemaInput);

  const name = schema.contract_name.replaceAll("cw-", "");
  const dir = join("types", name);

  const { lines } = await quicktype({
    inputData,
    lang: "go",
    rendererOptions: {
      package: name.replaceAll("-", ""),
    },
  });

  await mkdir(dir, { recursive: true });
  await writeFile(join(dir, "schema.go"), lines.join("\n"));
}

import cw_bvs_directory from "@satlayer/cw-bvs-directory/schema/cw-bvs-directory.json" with { type: "json" };
import cw_bvs_driver from "@satlayer/cw-bvs-driver/schema/cw-bvs-driver.json" with { type: "json" };
import cw_delegation_manager from "@satlayer/cw-delegation-manager/schema/cw-delegation-manager.json" with { type: "json" };
import cw_rewards_coordinator from "@satlayer/cw-rewards-coordinator/schema/cw-rewards-coordinator.json" with { type: "json" };
import cw_slash_manager from "@satlayer/cw-slash-manager/schema/cw-slash-manager.json" with { type: "json" };
import cw_state_bank from "@satlayer/cw-state-bank/schema/cw-state-bank.json" with { type: "json" };
import cw_strategy_base from "@satlayer/cw-strategy-base/schema/cw-strategy-base.json" with { type: "json" };
import cw_strategy_base_tvl_limits from "@satlayer/cw-strategy-base-tvl-limits/schema/cw-strategy-base-tvl-limits.json" with { type: "json" };
import cw_strategy_factory from "@satlayer/cw-strategy-factory/schema/cw-strategy-factory.json" with { type: "json" };
import cw_strategy_manager from "@satlayer/cw-strategy-manager/schema/cw-strategy-manager.json" with { type: "json" };

await generate(cw_bvs_directory);
await generate(cw_bvs_driver);
await generate(cw_delegation_manager);
await generate(cw_rewards_coordinator);
await generate(cw_slash_manager);
await generate(cw_state_bank);
await generate(cw_strategy_base);
await generate(cw_strategy_base_tvl_limits);
await generate(cw_strategy_factory);
await generate(cw_strategy_manager);

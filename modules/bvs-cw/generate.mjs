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

  const name = schema.contract_name.replaceAll("bvs-", "");
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

import bvs_delegation_manager from "@satlayer/bvs-delegation-manager/schema/bvs-delegation-manager.json" with { type: "json" };
import bvs_directory from "@satlayer/bvs-directory/schema/bvs-directory.json" with { type: "json" };
import bvs_driver from "@satlayer/bvs-driver/schema/bvs-driver.json" with { type: "json" };
import bvs_rewards_coordinator from "@satlayer/bvs-rewards-coordinator/schema/bvs-rewards-coordinator.json" with { type: "json" };
import bvs_slash_manager from "@satlayer/bvs-slash-manager/schema/bvs-slash-manager.json" with { type: "json" };
import bvs_state_bank from "@satlayer/bvs-state-bank/schema/bvs-state-bank.json" with { type: "json" };
import bvs_strategy_base from "@satlayer/bvs-strategy-base/schema/bvs-strategy-base.json" with { type: "json" };
import bvs_strategy_base_tvl_limits from "@satlayer/bvs-strategy-base-tvl-limits/schema/bvs-strategy-base-tvl-limits.json" with { type: "json" };
import bvs_strategy_factory from "@satlayer/bvs-strategy-factory/schema/bvs-strategy-factory.json" with { type: "json" };
import bvs_strategy_manager from "@satlayer/bvs-strategy-manager/schema/bvs-strategy-manager.json" with { type: "json" };

await generate(bvs_delegation_manager);
await generate(bvs_directory);
await generate(bvs_driver);
await generate(bvs_rewards_coordinator);
await generate(bvs_slash_manager);
await generate(bvs_state_bank);
await generate(bvs_strategy_base);
await generate(bvs_strategy_base_tvl_limits);
await generate(bvs_strategy_factory);
await generate(bvs_strategy_manager);

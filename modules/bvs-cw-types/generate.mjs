import { quicktype, InputData, JSONSchemaInput, FetchingJSONSchemaStore } from "quicktype-core";

import { mkdir, writeFile } from "node:fs/promises";

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

  const dir = schema.contract_name.replaceAll("cw-", "");

  const { lines } = await quicktype({
    inputData,
    lang: "go",
    rendererOptions: {
      package: dir.replaceAll("-", "_"),
    },
  });

  await mkdir(dir, { recursive: true });
  await writeFile(`${schema.contract_name.replaceAll("cw-", "")}/types.go`, lines.join("\n"));
}

import cw_bvs_driver from "@satlayer/cw-bvs-driver/schema/cw-bvs-driver.json" assert { type: "json" };
import cw_state_bank from "@satlayer/cw-state-bank/schema/cw-state-bank.json" assert { type: "json" };

await generate(cw_bvs_driver);
await generate(cw_state_bank);

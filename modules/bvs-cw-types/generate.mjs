import { quicktype, InputData, JSONSchemaInput, FetchingJSONSchemaStore } from "quicktype-core";

import { writeFile } from "node:fs/promises";

/**
 * @param schema {any}
 * @returns {Promise<void>}
 */
async function generate(schema) {
  const schemaInput = new JSONSchemaInput(new FetchingJSONSchemaStore());
  await schemaInput.addSource({ name: "InstantiateMsg", schema: JSON.stringify(schema.instantiate) });
  await schemaInput.addSource({ name: "ExecuteMsg", schema: JSON.stringify(schema.execute) });

  if (schema.query.enum.length !== 0) {
    await schemaInput.addSource({ name: "QueryMsg", schema: JSON.stringify(schema.query) });
  }
  const inputData = new InputData();
  inputData.addInput(schemaInput);

  const { lines } = await quicktype({
    inputData,
    lang: "go",
  });

  await writeFile(`${schema.contract_name}.go`, lines.join("\n"));
}

import cw_bvs_driver from "@satlayer/cw-bvs-driver/schema/cw-bvs-driver.json" assert { type: "json" };

void generate(cw_bvs_driver);

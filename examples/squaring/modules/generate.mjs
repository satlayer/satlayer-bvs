import { FetchingJSONSchemaStore, InputData, JSONSchemaInput, quicktype } from "quicktype-core";

import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

import squaring_contract from "@examples/squaring-contract/dist/schema.json" with { type: "json" };

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
  for (const [key, res] of Object.entries(schema.responses)) {
    await schemaInput.addSource({ name: key, schema: JSON.stringify(res) });
  }
  const inputData = new InputData();
  inputData.addInput(schemaInput);

  const name = schema.contract_name.replaceAll("bvs-", "");

  const { lines } = await quicktype({
    inputData,
    lang: "go",
    rendererOptions: {
      package: name.replaceAll("-", ""),
    },
  });

  await mkdir(name, { recursive: true });
  await writeFile(join(name, "schema.go"), lines.join("\n"));
}

await generate(squaring_contract);

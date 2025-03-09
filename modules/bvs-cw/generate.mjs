import { FetchingJSONSchemaStore, InputData, JSONSchemaInput, quicktype } from "quicktype-core";

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

const schemas = [
  "@satlayer/bvs-strategy-base/schema/bvs-strategy-base.json",
  "@satlayer/bvs-strategy-manager/schema/bvs-strategy-manager.json",
  "@satlayer/bvs-directory/schema/bvs-directory.json",
  "@satlayer/bvs-delegation-manager/schema/bvs-delegation-manager.json",
  "@satlayer/bvs-rewards-coordinator/schema/bvs-rewards-coordinator.json",
  "@satlayer/bvs-slash-manager/schema/bvs-slash-manager.json",

  "@satlayer/bvs-pauser/schema/bvs-pauser.json",
  "@satlayer/bvs-vault-router/schema/bvs-vault-router.json",
  "@satlayer/bvs-vault-cw20/schema/bvs-vault-cw20.json",
  "@satlayer/bvs-vault-bank/schema/bvs-vault-bank.json",
];

for (const schema of schemas) {
  const s = await import(schema, { with: { type: "json" } });
  await generate(s.default);
}

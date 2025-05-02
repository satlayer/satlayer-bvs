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
      "just-types": true,
    },
  });

  const content = [
    `// This file was automatically generated from ${name}/schema.json.`,
    "// DO NOT MODIFY IT BY HAND.",
    "",
    "package " + name.replaceAll("-", ""),
    ...lines,
  ];
  await mkdir(name, { recursive: true });
  await writeFile(join(name, "schema.go"), content.join("\n"));
}

const packages = [
  "@satlayer/bvs-pauser",
  "@satlayer/bvs-registry",
  "@satlayer/bvs-vault-router",
  "@satlayer/bvs-vault-cw20",
  "@satlayer/bvs-vault-cw20-tokenized",
  "@satlayer/bvs-vault-bank",
  "@satlayer/bvs-vault-factory",
  "@satlayer/bvs-rewards",
];

for (const schema of packages) {
  const s = await import(schema + "/dist/schema.json", { with: { type: "json" } });
  await generate(s.default);
}

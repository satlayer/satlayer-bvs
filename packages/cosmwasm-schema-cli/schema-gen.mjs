#!/usr/bin/env node

import { FetchingJSONSchemaStore, InputData, JSONSchemaInput, quicktype } from "quicktype-core";
import { mkdir, readFile, writeFile } from "fs/promises";
import { basename, dirname, join } from "path";

/**
 * Generate types from a schema file
 * @param {string} schemaPath - Path to the schema.json file
 * @param {string} outputDir - Optional output directory (defaults to current directory)
 * @param {string} language - Optional output language (defaults to 'go')
 */
async function generateTypesFromSchema(schemaPath, outputDir = ".", language = "go") {
  try {
    // Read and parse the schema file
    const schemaContent = await readFile(schemaPath, "utf8");
    const schema = JSON.parse(schemaContent);

    // Initialize quicktype inputs
    const schemaInput = new JSONSchemaInput(new FetchingJSONSchemaStore());

    // Add schema sources
    if (schema.instantiate) {
      await schemaInput.addSource({ name: "InstantiateMsg", schema: JSON.stringify(schema.instantiate) });
    }

    if (schema.execute) {
      await schemaInput.addSource({ name: "ExecuteMsg", schema: JSON.stringify(schema.execute) });
    }

    if (schema.query && (!schema.query.enum || schema.query.enum.length !== 0)) {
      await schemaInput.addSource({ name: "QueryMsg", schema: JSON.stringify(schema.query) });
    }

    // Process response schemas if they exist
    if (schema.responses) {
      for (const [key, res] of Object.entries(schema.responses)) {
        await schemaInput.addSource({ name: key, schema: JSON.stringify(res) });
      }
    }

    const inputData = new InputData();
    inputData.addInput(schemaInput);

    // Determine package/type name from schema or file path
    const name = schema.contract_name ?? basename(dirname(schemaPath));

    // Generate types with quicktype
    const { lines } = await quicktype({
      inputData,
      lang: language,
      rendererOptions: {
        "just-types": true,
      },
    });

    // Prepare output content
    let content;
    if (language === "go") {
      content = [
        `// This file was automatically generated from ${basename(schemaPath)}.`,
        "// DO NOT MODIFY IT BY HAND.",
        "",
        "package " + name.replaceAll("-", ""),
        ...lines,
      ].join("\n");
    } else {
      content = [
        `// This file was automatically generated from ${basename(schemaPath)}.`,
        "// DO NOT MODIFY IT BY HAND.",
        "",
        ...lines,
      ].join("\n");
    }

    // Create output directory and write file
    const outputPath = join(outputDir, `${name}.${getFileExtension(language)}`);
    await mkdir(dirname(outputPath), { recursive: true });
    await writeFile(outputPath, content);

    console.log(`Successfully generated ${outputPath}`);
  } catch (error) {
    console.error("Error generating types:", error);
    process.exit(1);
  }
}

/**
 * Get the appropriate file extension for the target language
 */
function getFileExtension(language) {
  const extensions = {
    go: "go",
    typescript: "ts",
  };

  return extensions[language] || "txt";
}

/**
 * Parse command line arguments and run the generator
 */
async function main() {
  const args = process.argv.slice(2);

  if (args.length === 0 || args.includes("--help") || args.includes("-h")) {
    console.log(`
Usage: schema-gen <schema-path> [options]

Options:
  --out-dir, -o    Output directory (default: current directory)
  --language, -l   Target language (default: typescript)
                   Supported: go, typescript
  --help, -h       Show this help message

Example:
  schema-gen ./path/to/schema.json --out-dir ./types --language typescript
`);
    process.exit(0);
  }

  const schemaPath = args[0];
  let outputDir = ".";
  let language = "typescript";

  // Parse options
  for (let i = 1; i < args.length; i++) {
    if (args[i] === "--out-dir" || args[i] === "-o") {
      outputDir = args[++i] || outputDir;
    } else if (args[i] === "--language" || args[i] === "-l") {
      language = args[++i] || language;
    }
  }

  await generateTypesFromSchema(schemaPath, outputDir, language);
}

// Run the program
main().catch(console.error);

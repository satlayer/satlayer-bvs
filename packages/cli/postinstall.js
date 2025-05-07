const { join } = require("node:path");
const { mkdirSync, existsSync, readFileSync } = require("node:fs");

if (existsSync(join(__dirname, "satlayer"))) {
  process.exit(0);
}

const bins = Object.keys(JSON.parse(readFileSync("package.json")).optionalDependencies);

for (const pkg of bins) {
  try {
    const binary = require.resolve(pkg + "/dist/satlayer");
    if (existsSync(binary)) {
      require("fs").symlinkSync(binary, join(__dirname, "satlayer"));
      process.exit(0);
    }
  } catch {}
}

console.error("No compatible binary found for your system.");
process.exit(1);

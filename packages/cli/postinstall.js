const { join } = require("node:path");
const { existsSync, symlinkSync } = require("node:fs");

if (existsSync(join(__dirname, "satlayer"))) {
  process.exit(0);
}

const packages = ["darwin-arm64", "darwin-x64", "linux-arm64", "linux-x64", "win32-arm64", "win32-x64"];

for (const pkg of packages) {
  try {
    const binary = require.resolve(`@satlayer/cli-${pkg}/satlayer`);
    if (existsSync(binary)) {
      symlinkSync(binary, join(__dirname, "satlayer"));
      process.exit(0);
    }
  } catch {}
}

console.error("No compatible binary found for your system.");
process.exit(1);

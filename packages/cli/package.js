const { cpSync } = require("node:fs");
const { join } = require("node:path");

const packages = ["darwin-arm64", "darwin-x64", "linux-arm64", "linux-x64", "win32-arm64", "win32-x64"];

for (const name of packages) {
  const binary = join("node_modules/@modules/cosmwasm-cli/dist", name);
  const target = join(`cli-${name}`, "satlayer");
  cpSync(binary, target);
}

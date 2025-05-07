import { cpSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const packages = ["darwin-arm64", "darwin-x64", "linux-arm64", "linux-x64", "win32-arm64", "win32-x64"];

for (const name of packages) {
  const [os, cpu] = name.split("-");

  const dir = `cli-${name}`;
  mkdirSync(dir);
  const packageJson = {
    name: `@satlayer/cli-${name}`,
    private: false,
    files: ["satlayer"],
    os: [os],
    cpu: [cpu],
  };

  writeFileSync(join(dir, "package.json"), JSON.stringify(packageJson, null, 2));

  const binary = join("node_modules/@modules/cosmwasm-cli/dist", name);
  const target = join(dir, "satlayer");
  cpSync(binary, target);
}

// Modify the package.json of the main package
const packageJson = JSON.parse(readFileSync("package.json").toString("utf-8"));
packageJson.scripts = {
  postinstall: "node postinstall.js",
};

packageJson.optionalDependencies = packages.reduce((acc, name) => {
  acc[`@satlayer/cli-${name}`] = `workspace:*`;
  return acc;
}, {});
writeFileSync("package.json", JSON.stringify(packageJson, null, 2));

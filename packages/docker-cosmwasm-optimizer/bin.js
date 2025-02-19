#!/usr/bin/env node

const { execSync } = require("child_process");
const path = require("path");

const cwd = process.cwd();
const crate = path.basename(cwd);
const parentDir = path.dirname(cwd);

const command = [
  "docker buildx build",
  "-f",
  path.join(path.dirname(require.main.filename), "Dockerfile"),
  `--output=./artifacts`,
  `--build-arg CRATE=${crate}`,
  parentDir,
];

if (process.env.CI) {
  command.push("--cache-from", `type=registry,ref=ghcr.io/satlayer/cosmwasm-optimizer-cache:${crate}`);
  command.push("--cache-to", `type=registry,ref=ghcr.io/satlayer/cosmwasm-optimizer-cache:${crate},mode=max`);
}

execSync(command.join(" "), { stdio: "inherit" });

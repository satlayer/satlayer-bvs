#!/usr/bin/env node

const { execSync } = require("child_process");
const path = require("path");

const cwd = process.cwd();
const projectName = path.basename(cwd);
const parentDir = path.dirname(cwd);

const baseCommand = ["docker run --rm", `-v ${parentDir}:/code`];

// Isn't much faster if we're using fresh instances on CI.
if (!process.env.CI) {
  baseCommand.push(`--mount type=volume,source=sl_bvs_cache_wasm_${projectName},target=/target`);
  baseCommand.push(`--mount type=volume,source=sl_bvs_cache_registry_${projectName},target=/usr/local/cargo/registry`);
}

// Compose final command
baseCommand.push(`ghcr.io/satlayer/cosmwasm-optimizer:local ./${projectName}`);

const command = baseCommand.join(" ");

execSync(command, { stdio: "inherit" });

#!/usr/bin/env node

const { execSync } = require("child_process");
const path = require("path");

const cwd = process.cwd();
const projectName = path.basename(cwd);
const parentDir = path.dirname(cwd);

const baseCommand = ["docker run --rm", `-v ${parentDir}:/code`];

// If running in CI, push the cache to the registry
if (process.env.CI) {
  baseCommand.push(`--cache-to=ghcr.io/satlayer/cosmwasm-optimizer-cache:${projectName}`);
  // TODO(fuxingloh): We can pull from the cache (but our repo is not public, so that will complicate things)
  baseCommand.push(`--cache-from=ghcr.io/satlayer/cosmwasm-optimizer-cache:${projectName}`);
} else {
  baseCommand.push(`--mount type=volume,source=sl_bvs_cache_wasm_${projectName},target=/target`);
  baseCommand.push(`--mount type=volume,source=sl_bvs_cache_registry_${projectName},target=/usr/local/cargo/registry`);
}

// Compose final command
baseCommand.push(`ghcr.io/satlayer/cosmwasm-optimizer:local ./${projectName}`);

const command = baseCommand.join(" ");

execSync(command, { stdio: "inherit" });

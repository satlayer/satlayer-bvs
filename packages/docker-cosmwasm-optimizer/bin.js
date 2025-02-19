#!/usr/bin/env node

const { execSync } = require("child_process");
const path = require("path");

const cwd = process.cwd();
const crate = path.basename(cwd);
const parentDir = path.dirname(cwd);

if (process.env.CI) {
  // TODO(fuxingloh): setup remote caching
}

const command = [
  "docker build",
  "-f",
  path.join(path.dirname(require.main.filename), "Dockerfile.builder"),
  `--output=./artifacts`,
  `--build-arg CRATE=${crate}`,
  parentDir,
].join(" ");

execSync(command, { stdio: "inherit" });

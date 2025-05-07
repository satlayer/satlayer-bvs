#!/usr/bin/env node

const { execSync } = require("node:child_process");
const { parseArgs } = require("node:util");
const path = require("node:path");

const options = {
  root: {
    type: "string",
  },
  name: {
    type: "string",
  },
};
const { values } = parseArgs({ options });

const cwd = process.cwd();
const rootDir = (values.root && path.join(cwd, values.root)) || cwd;
const name = values.name ?? path.basename(cwd);

function getDirectory() {
  if (values.root) {
    return "./" + path.basename(cwd);
  }
  return ".";
}

const command = [
  "docker buildx build",
  "-f",
  path.join(path.dirname(require.main.filename), "Dockerfile"),
  `--output=./dist`,
  // We only need to pass the directory if it is different from the current working directory
  `--build-arg DIRECTORY=${getDirectory()}`,
  rootDir,
];

if (process.env.DOCKER_CACHE_FROM) {
  const registry = process.env.DOCKER_CACHE_TO;
  command.push("--cache-from", `type=registry,ref=${registry}:${name}`);
}

if (process.env.DOCKER_CACHE_TO) {
  const registry = process.env.DOCKER_CACHE_TO;
  command.push("--cache-to", `type=registry,ref=${registry}:${name},mode=max`);
}

execSync(command.join(" "), { stdio: "inherit" });

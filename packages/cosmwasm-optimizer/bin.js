#!/usr/bin/env node

const { execSync } = require("node:child_process");
const { parseArgs } = require("node:util");
const path = require("node:path");

const cwd = process.cwd();
const options = {
  root: {
    type: "string",
  },
  dir: {
    type: "string",
  },
  output: {
    type: "string",
    default: "./dist",
  },
  name: {
    type: "string",
    default: path.basename(cwd),
  },
};
const { values } = parseArgs({ options });

const rootDir = (values.root && path.join(cwd, values.root)) || cwd;

function getDirectory() {
  if (values.dir) {
    return values.dir;
  }
  if (values.root) {
    return "./" + path.basename(cwd);
  }
  return ".";
}

const command = [
  "docker buildx build",
  "-f",
  path.join(path.dirname(require.main.filename), "Dockerfile"),
  `--output=${values.output}`,
  // We only need to pass the directory if it is different from the current working directory
  `--build-arg DIRECTORY=${getDirectory()}`,
  rootDir,
];

if (process.env.DOCKER_CACHE_FROM) {
  const registry = process.env.DOCKER_CACHE_FROM;
  command.push("--cache-from", `type=registry,ref=${registry}:${values.name}`);
}

if (process.env.DOCKER_CACHE_TO) {
  const registry = process.env.DOCKER_CACHE_TO;
  command.push("--cache-to", `type=registry,ref=${registry}:${values.name},mode=max`);
}

execSync(command.join(" "), { stdio: "inherit" });

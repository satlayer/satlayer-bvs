#!/usr/bin/env node

const { execFileSync } = require("child_process");

const PLATFORMS = ["darwin", "linux", "win32"];
const ARCHS = ["x64", "arm64"];

function getBinaryPath() {
  // Expected package name, e.g. @satlayer/cli-darwin-arm64/satlayer
  let { platform, arch } = process;
  const binary =
    PLATFORMS.includes(platform) && ARCHS.includes(arch) ? `@satlayer/cli-${platform}-${arch}/satlayer` : null;
  if (binary) {
    try {
      return require.resolve(binary);
    } catch {}
  }

  // Rosetta / WOW64 fall-back
  const altArch = arch === "arm64" ? "x64" : null;
  const altBinary = altArch && PLATFORMS.includes(platform) ? `@satlayer/cli-${platform}-${altArch}/satlayer` : null;

  if (altBinary) {
    try {
      console.warn(`Falling back to ${altBinary}`);
      return require.resolve(altBinary);
    } catch {}
  }

  console.error(`No binary for ${platform} ${arch} found in node_modules.`);
  process.exit(1);
}

try {
  execFileSync(getBinaryPath(), process.argv.slice(2), { stdio: "inherit" });
} catch (e) {
  if (e && e.status) process.exit(e.status);
  throw e;
}

import { cpSync, readFileSync, rmSync } from "node:fs";
import { join } from "node:path";

const packageJson = JSON.parse(readFileSync("./package.json", "utf8"));

rmSync("lib", {
  recursive: true,
  force: true,
});

for (const pkg of Object.keys(packageJson.dependencies)) {
  if (pkg.startsWith("@")) {
    const [scope, name] = pkg.split("/");
    cpSync(join("node_modules", scope, name), join("lib", scope.replace("@", "") + "-" + name));
  } else {
    cpSync(join("node_modules", pkg), join("lib", pkg));
  }
}

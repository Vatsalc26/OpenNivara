import fs from "node:fs";
import path from "node:path";

const roots = ["docs", "docs-site"];
const linkPattern = /\[[^\]]+\]\(([^)]+)\)/g;
const failures = [];

for (const root of roots) {
  for (const file of walk(root)) {
    if (!file.endsWith(".md") && !file.endsWith(".mdx")) continue;
    const text = fs.readFileSync(file, "utf8");
    for (const match of text.matchAll(linkPattern)) {
      const rawTarget = match[1].trim();
      if (
        rawTarget.startsWith("http://") ||
        rawTarget.startsWith("https://") ||
        rawTarget.startsWith("mailto:") ||
        rawTarget.startsWith("#")
      ) {
        continue;
      }
      const [targetPath] = rawTarget.split("#");
      if (!targetPath) continue;
      const resolved = path.resolve(path.dirname(file), targetPath);
      if (!fs.existsSync(resolved)) {
        failures.push(`${file}: missing link target ${rawTarget}`);
      }
    }
  }
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("Internal docs links ok");

function* walk(dir) {
  if (!fs.existsSync(dir)) return;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (["node_modules", "target", "dist"].includes(entry.name)) continue;
      yield* walk(fullPath);
    } else {
      yield fullPath;
    }
  }
}

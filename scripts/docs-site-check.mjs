import fs from "node:fs";
import path from "node:path";

const siteRoot = fs.existsSync(path.resolve("docs.json"))
  ? path.resolve(".")
  : path.resolve("docs-site");
const configPath = path.join(siteRoot, "docs.json");
const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
const pages = [];

for (const tab of config.navigation?.tabs ?? []) {
  for (const group of tab.groups ?? []) {
    pages.push(...(group.pages ?? []));
  }
}

const missing = pages.filter((page) => {
  const mdxPath = path.join(siteRoot, `${page}.mdx`);
  return !fs.existsSync(mdxPath);
});

if (!fs.existsSync(path.join(siteRoot, "index.mdx"))) {
  missing.push("index");
}

if (missing.length > 0) {
  console.error(`Missing docs-site pages:\n${missing.join("\n")}`);
  process.exit(1);
}

console.log(`Mintlify docs-site structure ok (${pages.length} navigation pages)`);

import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

const root = process.cwd();
const skippedDirectories = new Set([
  ".git",
  "target",
  "node_modules",
  "dist",
  "build",
  ".next",
  ".turbo",
  ".cache",
]);
const skippedFiles = new Set(["desktop/bun.lock"]);

const forbidden = [
  ["scripts", "generate_india_skill", "_packs.py"].join("/"),
  ["generate_india_skill", "_packs.py"].join(""),
  ["OPENNIVARA", "REGENERATE_CURATED_STUDENT_ESSENTIALS"].join("_"),
  ["generated exam", "-pack manifests"].join(""),
  ["regenerate the generated India", " exam packs"].join(""),
  ["Python skill", " generator"].join(""),
];

const deletedPath = path.join(
  root,
  "scripts",
  ["generate_india_skill", "_packs.py"].join(""),
);

async function exists(filePath) {
  try {
    await readFile(filePath);
    return true;
  } catch (error) {
    if (error.code === "ENOENT") return false;
    throw error;
  }
}

async function* walk(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (entry.isDirectory() && skippedDirectories.has(entry.name)) continue;
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      yield* walk(fullPath);
    } else if (entry.isFile()) {
      yield fullPath;
    }
  }
}

const failures = [];

if (await exists(deletedPath)) {
  failures.push("obsolete skill-pack generator file exists");
}

for await (const filePath of walk(root)) {
  const relativePath = path.relative(root, filePath).replaceAll("\\", "/");
  if (relativePath === "scripts/check-no-obsolete-skill-generator.mjs") continue;
  if (skippedFiles.has(relativePath)) continue;

  let text;
  try {
    text = await readFile(filePath, "utf8");
  } catch {
    continue;
  }

  for (const phrase of forbidden) {
    if (text.includes(phrase)) {
      failures.push(`${relativePath}: contains obsolete skill generator reference`);
    }
  }
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("No obsolete skill generator references found.");

import { readdir, readFile, mkdir, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import path from "node:path";
const root = fileURLToPath(new URL("../../../", import.meta.url));
const files = [];
async function walk(relative) {
  for (const entry of await readdir(path.join(root, relative), {
    withFileTypes: true,
  })) {
    const name = `${relative}/${entry.name}`;
    if (entry.isDirectory()) await walk(name);
    else if (/\.(rs|js|py|sh|ps1)$/.test(name)) {
      const text = await readFile(path.join(root, name), "utf8");
      const symbols = [
        ...text.matchAll(/\b(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)/g),
      ].map((m) => m[1]);
      files.push({
        path: name,
        lines: text.split("\n").length,
        hash: createHash("sha256").update(text).digest("hex").slice(0, 12),
        symbols: [...new Set(symbols)],
        test: /(?:\/tests?\/|tests?\.(rs|py|js)$|\/fixtures\/)/.test(name),
      });
    }
  }
}
for (const entry of await readdir(path.join(root, "crates"), {
  withFileTypes: true,
})) {
  if (entry.isDirectory()) {
    await walk(`crates/${entry.name}/src`);
    try {
      await walk(`crates/${entry.name}/tests`);
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
    }
  }
}
await walk("apps/server/src");
await walk("apps/server/tests");
await walk("scripts");
files.sort((a, b) => a.path.localeCompare(b.path));
const crates = [];
for (const name of [
  ...new Set(
    files
      .filter((f) => f.path.startsWith("crates/"))
      .map((f) => f.path.split("/")[1]),
  ),
]) {
  const manifest = await readFile(
    path.join(root, "crates", name, "Cargo.toml"),
    "utf8",
  );
  crates.push({
    name,
    dependencies: [...manifest.matchAll(/^(fenix-[\w-]+)\s*=/gm)].map(
      (m) => m[1],
    ),
  });
}
const directory = fileURLToPath(new URL("../src/data/", import.meta.url));
await mkdir(directory, { recursive: true });
await writeFile(
  path.join(directory, "atlas-inventory.json"),
  JSON.stringify({ files, crates }, null, 2) + "\n",
);
console.log(
  `Atlas: ${files.length} backend source/test files, ${crates.length} crates.`,
);

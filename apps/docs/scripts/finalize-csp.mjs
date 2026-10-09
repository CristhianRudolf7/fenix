import { createHash } from "node:crypto";
import { readFile, readdir, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

// Account for Starlight's is:inline scripts using their exact emitted bytes.
// This runs only on trusted build output, never on user-uploaded documents.
const output = fileURLToPath(new URL("../dist/", import.meta.url));
let checked = 0;
async function visit(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const filename = `${directory}/${entry.name}`;
    if (entry.isDirectory()) await visit(filename);
    else if (entry.name.endsWith(".html")) {
      let html = await readFile(filename, "utf8");
      const hashes = new Set();
      for (const [, attributes, content] of html.matchAll(
        /<script\b([^>]*)>([\s\S]*?)<\/script>/g,
      )) {
        if (/\bsrc\s*=/.test(attributes)) continue;
        hashes.add(
          `'sha256-${createHash("sha256").update(content).digest("base64")}'`,
        );
      }
      let policyFound = false;
      html = html.replace(
        /(<meta\s+http-equiv="content-security-policy"\s+content=")([^"]+)(")/,
        (_, before, policy, after) => {
          policyFound = true;
          return (
            before +
            policy.replace(/script-src ([^;]+);/, (directive, resources) => {
              const missing = [...hashes].filter(
                (hash) => !resources.includes(hash),
              );
              return `script-src ${resources}${missing.length ? " " + missing.join(" ") : ""};`;
            }) +
            after
          );
        },
      );
      if (!policyFound) throw new Error(`Missing CSP in ${filename}`);
      await writeFile(filename, html);
      checked++;
    }
  }
}
await visit(output);
console.log(`Verified exact inline-script hashes on ${checked} pages.`);

// Collect every results file from ../results into a single module the site
// imports at build time. The site has no backend and never fetches at runtime.
//
// Missing data is never invented here. If a directory is empty, the generated
// payload says so and the UI renders "not yet measured".

import { readdirSync, readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(here, "..", "..");
const resultsRoot = join(repoRoot, "results");
const outDir = join(here, "..", "src", "generated");
const outFile = join(outDir, "results.json");

function loadKind(kind) {
  const dir = join(resultsRoot, kind);
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((f) => f.endsWith(".json"))
    .sort()
    .map((f) => {
      const raw = readFileSync(join(dir, f), "utf8");
      try {
        const parsed = JSON.parse(raw);
        parsed._file = `results/${kind}/${f}`;
        return parsed;
      } catch (err) {
        throw new Error(`Malformed results file ${kind}/${f}: ${err.message}`);
      }
    });
}

const native = loadKind("native");
const zkvm = loadKind("zkvm");

mkdirSync(outDir, { recursive: true });
writeFileSync(
  outFile,
  JSON.stringify(
    {
      collected_at: new Date().toISOString(),
      native,
      zkvm,
    },
    null,
    2
  ) + "\n"
);

console.log(
  `collect-results: ${native.length} native run(s), ${zkvm.length} zkvm run(s) -> src/generated/results.json`
);

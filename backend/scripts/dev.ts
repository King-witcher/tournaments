// Sobe a API e reinicia a cada mudança no backend, como um --watch.
// Uso, em backend/: bun scripts/dev.ts
//
// A cada mudança em .rs, .sql, Cargo.toml ou .env: encerra a API, aplica as
// migrations (as macros query! conferem o SQL contra o banco na compilação),
// compila e sobe de novo. Se algo falhar, espera a próxima mudança.
// Respeita CARGO_TARGET_DIR e as variáveis da API (ex.: API_ADDR).

import { statSync, watch } from "node:fs";
import { join } from "node:path";

const root = join(import.meta.dir, "..");
const targetDir = process.env.CARGO_TARGET_DIR ?? join(root, "target");
const exe = join(
  targetDir,
  "debug",
  process.platform === "win32" ? "api.exe" : "api",
);
const relevant = /\.(rs|toml)$/;

let api: ReturnType<typeof Bun.spawn> | undefined;
let building = false;
let dirty = false;
let timer: ReturnType<typeof setTimeout> | undefined;

function cargo(...args: string[]) {
  return Bun.spawn(["cargo", ...args], {
    cwd: root,
    stdio: ["inherit", "inherit", "inherit"],
  }).exited;
}

async function stopApi() {
  if (!api) return;
  api.kill();
  await api.exited;
  api = undefined;
}

async function rebuild() {
  if (building) {
    dirty = true;
    return;
  }
  building = true;
  do {
    dirty = false;
    // A API precisa parar antes do build: no Windows o .exe em uso fica travado.
    await stopApi();
    console.log("\n[dev] migrations e build…");
    const ok =
      (await cargo("run", "-q", "-p", "migrate")) === 0 &&
      (await cargo("build", "-p", "api")) === 0;
    if (dirty) continue; // mudou durante o build: recomeça
    if (ok) {
      api = Bun.spawn([exe], {
        cwd: root,
        stdio: ["inherit", "inherit", "inherit"],
      });
    } else {
      console.log("[dev] falhou; esperando a próxima mudança.");
    }
  } while (dirty);
  building = false;
}

// No Windows o fs.watch também avisa quando um arquivo só foi lido (data de
// último acesso), por exemplo o .env quando a API sobe. Só conta como mudança
// se a data de modificação mudou.
const startedAt = Date.now();
const mtimes = new Map<string, number>();

function modified(path: string) {
  let mtime: number;
  try {
    mtime = statSync(path).mtimeMs;
  } catch {
    return true; // apagado ou renomeado
  }
  const previous = mtimes.get(path);
  mtimes.set(path, mtime);
  return previous === undefined ? mtime > startedAt : mtime !== previous;
}

function watchDir(dir: string, accept: (file: string) => boolean, recursive = false) {
  watch(dir, { recursive }, (_event, file) => {
    if (!file || !accept(file) || !modified(join(dir, file))) return;
    console.log(`[dev] mudou: ${file}`);
    clearTimeout(timer);
    timer = setTimeout(rebuild, 300);
  });
}

watchDir(join(root, "crates"), (f) => relevant.test(f), true);
watchDir(join(root, "migrations"), (f) => f.endsWith(".sql"));
watchDir(root, (f) => f === "Cargo.toml" || f === ".env");

process.on("SIGINT", async () => {
  await stopApi();
  process.exit(0);
});

rebuild();

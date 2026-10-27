// Gera backend/.sqlx, os dados das queries que deixam compilar sem banco
// (SQLX_OFFLINE=true, como no build do Render).
// Uso, em backend/: bun scripts/sqlx-prepare.ts
//
// Rode depois de criar ou mudar qualquer query! e faça commit do .sqlx/; sem
// isso o build sem banco falha com "no cached data for this query".
// Faz o que o `cargo sqlx prepare` faz, sem o sqlx-cli: com SQLX_OFFLINE_DIR
// definido, as próprias macros gravam um arquivo por query ao compilar contra
// o banco local. Compila numa pasta de target própria para não esbarrar no
// api.exe travado da API rodando.

import { mkdirSync, readdirSync, rmSync } from "node:fs";
import { join } from "node:path";

const root = join(import.meta.dir, "..");
const offlineDir = join(root, ".sqlx");
const env = {
  ...process.env,
  CARGO_TARGET_DIR: join(root, "target", "sqlx-prepare"),
  SQLX_OFFLINE: "false",
};

function cargo(args: string[], extra: Record<string, string> = {}) {
  return Bun.spawn(["cargo", ...args], {
    cwd: root,
    env: { ...env, ...extra },
    stdio: ["inherit", "inherit", "inherit"],
  }).exited;
}

// Recomeça do zero para não sobrar arquivo de query que já não existe.
rmSync(offlineDir, { recursive: true, force: true });
mkdirSync(offlineDir);

const ok =
  (await cargo(["run", "-q", "-p", "migrate"])) === 0 &&
  // Sem limpar, o cargo reaproveita a compilação anterior e as macros não rodam.
  (await cargo(["clean", "-q", "-p", "api"])) === 0 &&
  (await cargo(["check", "-p", "api", "--all-targets"], {
    SQLX_OFFLINE_DIR: offlineDir,
  })) === 0;

if (!ok) {
  console.error("[sqlx-prepare] falhou; o .sqlx/ ficou incompleto.");
  process.exit(1);
}
console.log(`[sqlx-prepare] ${readdirSync(offlineDir).length} queries em .sqlx/`);

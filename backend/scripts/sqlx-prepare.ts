// Gera backend/.sqlx, os dados das queries que deixam compilar sem banco
// (SQLX_OFFLINE=true, como no build do Render).
// Uso, em backend/: bun scripts/sqlx-prepare.ts
//
// Rode depois de criar ou mudar qualquer query! e faça commit do .sqlx/; sem
// isso o build sem banco falha com "no cached data for this query".
// Faz o que o `cargo sqlx prepare` faz, sem o sqlx-cli: com SQLX_OFFLINE_DIR
// definido, as próprias macros gravam um arquivo por query ao compilar contra
// o banco. Compila numa pasta de target própria para não esbarrar no
// api.exe travado da API rodando.
//
// O banco é novo a cada vez, só com as migrations: o sqlx deduz a nulidade das
// colunas do plano da query (EXPLAIN), que muda com os dados. Sem dados, o
// resultado é o mesmo em qualquer máquina e igual ao que a CI compara.

import { SQL } from "bun";
import { mkdirSync, readdirSync, rmSync } from "node:fs";
import { join } from "node:path";

const root = join(import.meta.dir, "..");
const offlineDir = join(root, ".sqlx");

// `telefrag` → `_sqlx_test_telefrag_prepare`, com o prefixo que o pg_hba.conf do
// README libera para o usuário do projeto, como os bancos de teste. Fica até a
// próxima vez: o drop do fim custaria mais um checkpoint.
const url = new URL(process.env.DATABASE_URL ?? "postgres://telefrag@localhost:5432/telefrag");
const fresh = `_sqlx_test_${url.pathname.slice(1).replace(/^_sqlx_test_/, "")}_prepare`;
const admin = new SQL(url.href);
await admin.unsafe(`drop database if exists "${fresh}" with (force)`);
await admin.unsafe(`create database "${fresh}"`);
await admin.close();
url.pathname = `/${fresh}`;

const env = {
  ...process.env,
  CARGO_TARGET_DIR: join(root, "target", "sqlx-prepare"),
  DATABASE_URL: url.href,
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

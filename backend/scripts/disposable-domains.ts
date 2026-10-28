// Atualiza a lista de domínios de email descartáveis que a API recusa
// (crates/api/src/mail/disposable_domains.txt), a partir da lista aberta do
// projeto disposable-email-domains (CC0). Rodar em backend/:
//   bun scripts/disposable-domains.ts
// e fazer commit do arquivo. A lista entra no binário na compilação.

const SOURCE =
  "https://raw.githubusercontent.com/disposable-email-domains/disposable-email-domains/main/disposable_email_blocklist.conf";
const TARGET = "crates/api/src/mail/disposable_domains.txt";

const res = await fetch(SOURCE);
if (!res.ok) {
  throw new Error(`download falhou: ${res.status} ${res.statusText}`);
}
const domains = [
  ...new Set(
    (await res.text())
      .split("\n")
      .map((line) => line.trim().toLowerCase())
      .filter((line) => line !== "" && !line.startsWith("#")),
  ),
].sort();

const invalid = domains.filter((d) => !/^[a-z0-9-]+(\.[a-z0-9-]+)+$/.test(d));
if (invalid.length > 0) {
  throw new Error(`linhas fora do formato de domínio: ${invalid.join(", ")}`);
}

const header = [
  "# Domínios de email descartáveis, recusados no cadastro e na troca de email.",
  "# Fonte: disposable-email-domains/disposable-email-domains no GitHub (CC0).",
  "# Atualizar em backend/: bun scripts/disposable-domains.ts",
];
await Bun.write(TARGET, `${[...header, ...domains].join("\n")}\n`);
console.log(`${domains.length} domínios em ${TARGET}`);

@AGENTS.md

# Frontend (web/)

- Só apresentação: nunca acessa o banco. Os dados vêm da API Rust pelo client `api` em `src/lib/api/client.ts` (openapi-fetch).
- Gerenciador de pacotes e scripts: Bun (`bun add`, `bun run ...`). Lint e formatação: Biome (`bun run lint`, `bun run format`), não ESLint/Prettier.
- `src/lib/api/schema.d.ts` é gerado: não edite à mão. Com a API rodando, rode `bun run gen:api`.
- Todo texto visível vem de `messages/pt.json` e `messages/es.json` (next-intl). Nada de texto direto no JSX; toda chave nova entra nos dois arquivos.
- Códigos que a API devolve (status, erros) são traduzidos aqui.
- O idioma é o segmento raiz `[locale]`. No servidor, `getTranslations` já lê o idioma de `next/root-params`; não passe locale à mão.
- Links e redirects pelo `src/i18n/navigation.ts`, não por `next/link`/`next/navigation`, para manter o prefixo de idioma.
- Cache Components está ligado: dado buscado a cada requisição fica dentro de `<Suspense>`.

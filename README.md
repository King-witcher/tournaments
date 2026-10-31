# Torneios de Quake Champions

Site multi-tenant para organizar torneios de Quake Champions. API em Rust (Axum + sqlx), frontend em Next.js. Decisões do projeto em [`docs/decisions.md`](docs/decisions.md) (índice por área).

## Pré-requisitos

- PostgreSQL 18 local
- Redis local (no Windows, o [Memurai](https://www.memurai.com/), compatível; ou `docker run -d -p 6379:6379 redis`)
- Rust (stable)
- Bun e Node 24 (o Bun instala e roda os scripts; o Next roda em Node, igual à Vercel)

## Banco local

O projeto usa um banco `tournaments` com um usuário `tournaments` que só acessa esse banco e entra sem senha a partir desta máquina. Para criar numa máquina nova, como `postgres`:

```sql
CREATE ROLE tournaments LOGIN NOSUPERUSER NOCREATEROLE CREATEDB NOREPLICATION;
CREATE DATABASE tournaments OWNER tournaments;
REVOKE ALL ON DATABASE tournaments FROM PUBLIC;
GRANT CONNECT, TEMPORARY ON DATABASE tournaments TO tournaments;
```

`CREATEDB` existe só para os testes: eles criam até 8 bancos `_sqlx_test_<banco>_t<n>` (`backend/crates/api/tests/api/db.rs`), mantidos entre execuções e esvaziados antes de cada teste.

No `pg_hba.conf`, **antes** das linhas `all all`, e depois `SELECT pg_reload_conf();`:

```
host    tournaments,/^_sqlx_test    tournaments    127.0.0.1/32    trust
host    tournaments,/^_sqlx_test    tournaments    ::1/128         trust
local   tournaments,/^_sqlx_test    tournaments                    trust
host    all                         tournaments    all             reject
local   all                         tournaments                    reject
```

As três primeiras liberam o usuário sem senha só no próprio banco e nos de teste; as duas últimas recusam o resto.

## Redis local

Sessões e outros dados efêmeros ficam no Redis, sempre no database 1 (`redis://127.0.0.1:6379/1` no `.env.example`; os outros databases são de outros projetos). A API põe `REDIS_PREFIX` (padrão `tournaments:`) em toda chave, o que separa os ambientes deste projeto (um worktree com outro prefixo não vê as sessões do checkout principal). Os testes usam o mesmo database com um prefixo `test:` por teste (ver `docs/decisions/stack.md`, "Redis").

## Rodando

Na primeira vez:

```sh
cp backend/.env.example backend/.env
cp web/.env.example web/.env.local
cd web && bun install
```

Os provedores de login voltam para a página de callback do front, `{WEB_ORIGIN}/auth/{discord|google}/callback` (sem idioma), que é o redirect a registrar em cada um; a API monta o endereço a partir do `WEB_ORIGIN`, e o front, a partir da própria origem (as duas precisam ser a mesma). Para entrar, o botão vai direto ao provedor com o client id público, que o front lê de `NEXT_PUBLIC_DISCORD_CLIENT_ID` e `NEXT_PUBLIC_GOOGLE_CLIENT_ID` (`web/.env.local`; em produção, nas variáveis do front, antes do build): o mesmo valor do `*_CLIENT_ID` da API, senão o login falha na volta.

Para o login, crie uma aplicação em https://discord.com/developers/applications, adicione o redirect `http://localhost:4001/auth/discord/callback` em **OAuth2 → Redirects** e coloque `DISCORD_CLIENT_ID` e `DISCORD_CLIENT_SECRET` no `backend/.env` e o mesmo client id em `NEXT_PUBLIC_DISCORD_CLIENT_ID` no `web/.env.local`. Sem eles, tudo funciona menos o login com Discord (o login com email e senha não depende dele).

O login com Google usa um client OAuth do Google Cloud (tipo "Aplicativo da Web", escopos `openid` e `profile`), com `GOOGLE_CLIENT_ID` e `GOOGLE_CLIENT_SECRET` na API e `NEXT_PUBLIC_GOOGLE_CLIENT_ID` no front. Sem eles, o login com Google diz que não está configurado. O client atual só tem o redirect de produção: localmente, com as variáveis, o Google recusa o redirect; para testar aqui, acrescente `http://localhost:4001/auth/google/callback` aos redirects autorizados do client (ou crie um só para dev). Em produção, as duas variáveis no Render, o client id também no front e o redirect do front de lá no client.

Os emails (confirmação do cadastro, recuperação de senha, troca de email) saem pelo Resend. Localmente, sem `RESEND_API_KEY` e `EMAIL_FROM`, a API só escreve cada email no log, com o link, e o fluxo funciona copiando o link de lá. Para mandar de verdade, crie uma API key com permissão só de envio no painel do Resend (domínio já verificado) e ponha as duas variáveis no `backend/.env`; em produção, no Render (lá, com `COOKIE_SECURE=true`, a API não sobe sem elas).

Para os avisos fora do site (Web Push), gere as chaves VAPID com `bun scripts/vapid-keys.ts` (em `backend/`, sem instalar nada) e cole as duas linhas que ele imprime (`VAPID_PUBLIC_KEY` e `VAPID_PRIVATE_KEY`) no `backend/.env`; `VAPID_SUBJECT` (um `mailto:` ou `https:`) é opcional e, sem ele, vai a origem do front. Sem as chaves, tudo funciona e os avisos só chegam com o site aberto, pelo WebSocket. Em produção, as mesmas variáveis no Render: trocar o par depois faz cada navegador precisar ativar os avisos de novo.

Os avisos da comunidade no Discord não precisam de variável de ambiente nem de bot: o owner cria um webhook no canal e cola o endereço nas configurações da comunidade. No Discord: **Configurações do servidor → Integrações → Webhooks → Novo webhook**, escolha o canal, copie a URL do webhook (`https://discord.com/api/webhooks/…`). No site: aba **Configurações** da comunidade (só o owner a vê), seção "Avisos no Discord"; cole a URL, escolha os avisos e o idioma e use "Enviar teste". A URL é uma credencial (a API nunca a devolve nem a escreve no log): quem a perde apaga o webhook no Discord e cola outro.

Os screenshots do placar vão para um bucket S3 (ver "Bucket dos screenshots" abaixo). Localmente, deixe as variáveis `S3_*` de fora: tudo funciona, os screenshots somem da interface (o "screenshot obrigatório" do torneio não liga) e nada vai para o bucket de produção.

Depois, em um terminal, a API com recarga automática: a cada `.rs`, `.sql`, `Cargo.toml` ou `.env` salvo, ela para, aplica as migrations, recompila e sobe de novo. Se a compilação falhar, espera a próxima mudança.

```sh
cd backend
bun scripts/dev.ts
```

Sem recarga, `cargo run -p api` sobe uma vez (e também aplica as migrations).

- API: http://localhost:4000
- Docs (Scalar): http://localhost:4000/docs

Em outro terminal, o frontend:

```sh
cd web
bun run dev
```

- Site: http://localhost:4001 (redireciona para `/pt` ou `/es` conforme o navegador)

## Bucket dos screenshots

Os screenshots vão do navegador direto para o bucket, por URLs assinadas que a API gera (o navegador nunca vê as chaves); a API só confere o arquivo (HEAD) e apaga (os removidos, os de torneios encerrados há mais de 30 dias e os uploads abandonados, numa rotina diária). Qualquer S3 compatível serve; em produção, AWS S3:

No console, cada screenshot fica em `screenshots/<comunidade>/<torneio>/<partida>/mapa-<n>-<aleatório>.webp` (ex.: `screenshots/quake-br/copa/wb-1-3/mapa-2-….webp`), as mesmas pastas do site. Um bucket por ambiente: não há prefixo de ambiente (motivo em `docs/decisions/screenshots.md`).

1. **Bucket**: no console do S3, crie um bucket na região da API (ex.: `us-east-1`), classe Standard, com **Block all public access ligado** (os screenshots só saem por URL assinada) e o versionamento desligado (apagar precisa apagar de vez).
2. **CORS** do bucket (aba Permissions → CORS), liberando o PUT do navegador pela origem do front (o endereço do site em produção, o mesmo `WEB_ORIGIN` da API, sem barra no fim). É o que falta configurar no bucket além das variáveis:

   ```json
   [
     {
       "AllowedOrigins": ["https://seu-front.exemplo"],
       "AllowedMethods": ["PUT"],
       "AllowedHeaders": ["content-type", "if-none-match"],
       "MaxAgeSeconds": 3600
     }
   ]
   ```

   O `if-none-match` é o que faz cada URL de upload gravar uma vez só (sem ele no CORS, todo envio falha). As imagens aparecem em `<img>`, que não precisa de CORS.
3. **Usuário IAM** só para a API, com uma access key e esta política (troque `NOME-DO-BUCKET`):

   ```json
   {
     "Version": "2012-10-17",
     "Statement": [
       {
         "Effect": "Allow",
         "Action": ["s3:PutObject", "s3:GetObject", "s3:DeleteObject"],
         "Resource": "arn:aws:s3:::NOME-DO-BUCKET/screenshots/*"
       }
     ]
   }
   ```

   `PutObject` assina o envio do navegador, `GetObject` a leitura (e o HEAD da conferência) e `DeleteObject` a exclusão (inclusive em lote).
4. **Variáveis** que a API lê (no Render, e no `backend/.env` só se quiser um bucket de dev à parte): `S3_BUCKET` (o nome do bucket), `S3_ACCESS_KEY_ID` e `S3_SECRET_ACCESS_KEY` (a access key do usuário IAM) e `S3_REGION` (a região do bucket; sem ela, `us-east-1`). Com só parte das três primeiras, a API não sobe (é engano de configuração); com nenhuma, sobe sem screenshots. `S3_ENDPOINT` só para outro provedor compatível (Cloudflare R2, MinIO), com o bucket no caminho da URL.
5. **Conferir**: com as variáveis no Render, o log da API não mostra o aviso "screenshots desligados" e `GET /screenshots/settings` responde `"enabled": true`. Sem o CORS do passo 2, o PUT do navegador falha e a tela mostra "O envio da imagem falhou".

## Comandos e CI

Os comandos do dia a dia (API com recarga, testes, clippy, migrations, tipos do front) estão no `CLAUDE.md`. O `.github/workflows/ci.yml` roda em push na `main` e em pull request, backend e web em paralelo, só o lado que mudou nos pull requests: `cargo fmt --check`, `cargo clippy` offline, migrations, `cargo test` (Postgres 18 e Redis como serviços) e a conferência de que o `.sqlx/` versionado é o que as queries geram (se falhar, `bun scripts/sqlx-prepare.ts` em `backend/` e commit); no web, `bun install --frozen-lockfile`, `bun run lint` e `bun run build`. A versão do Bun está fixa no workflow: ao subir o Bun localmente, suba lá também.

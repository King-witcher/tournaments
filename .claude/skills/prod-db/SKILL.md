---
name: prod-db
description: Consulta o banco de produção (tournaments_production, Postgres no Supabase) só para leitura, via psql com o usuário claude_readonly. Use quando o usuário pedir dados, contagens, estado de um torneio ou diagnóstico de algo em produção. Não serve para alterar dados nem para o banco local.
allowed-tools: Bash(psql:*)
---

# Banco de produção (somente leitura)

Com o Bash tool:

```bash
psql "host=$PROD_DB_HOST port=$PROD_DB_PORT dbname=$PROD_DB_NAME user=$PROD_DB_USER password=$PROD_DB_PASSWORD sslmode=$PROD_DB_SSLMODE" -X -c "select count(*) from tournaments"
```

As variáveis vêm do `env` de `.claude/settings.local.json`, fora do git. Se estiverem vazias, peça para o usuário conferir o arquivo.

- O usuário lê tudo do schema `public` e não escreve: escrita falha com `permission denied`, não tente contornar. Consultas param em 30 s.
- O schema está em `backend/migrations/`; `\dt` e `\d tabela` também funcionam. Use `limit` em tabela grande.
- São dados de pessoas reais: resuma o que importa e não copie para arquivos do repositório, testes ou commits. Nunca mostre a senha.

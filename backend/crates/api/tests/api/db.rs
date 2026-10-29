//! O banco de cada teste. Cada banco principal (`DATABASE_URL`) tem poucos
//! bancos de teste (`TEST_DATABASES`, padrão 4), criados e migrados uma vez e
//! mantidos entre execuções; antes de cada teste, o que o anterior gravou
//! sai. Criar e apagar um banco por teste, como faz o `sqlx::test`, custava
//! um checkpoint do Postgres a cada teste, e com outras suítes rodando cada
//! teste esperava minutos. Pelo mesmo motivo, uma suíte espera a de outra
//! sessão terminar (lock de arquivo na pasta temporária).

use std::{
    collections::HashMap,
    fs::{File, TryLockError},
    io::Write,
    sync::{
        LazyLock, Mutex, OnceLock, PoisonError,
        atomic::{AtomicU64, Ordering},
    },
};

use sqlx::{
    AssertSqlSafe, Connection, PgConnection, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use tokio::sync::{OnceCell, Semaphore, SemaphorePermit};

/// Bancos de teste, e testes de banco ao mesmo tempo, sem `TEST_DATABASES`.
const DEFAULT_DATABASES: usize = 4;

/// Conexões do pool de cada teste (o app e as consultas do teste dividem).
const CONNECTIONS: u32 = 8;

/// Onde ficam as linhas que as migrations gravam (campeões, mapas), para
/// voltarem a cada teste.
const SEED_SCHEMA: &str = "test_seed";

/// Um banco migrado e vazio, só deste teste enquanto o `Lease` existir:
/// `let (db, _lease) = crate::db::test_db().await;`.
pub async fn test_db() -> (PgPool, Lease) {
    LazyLock::force(&SUITE_LOCK);
    let slots = &*SLOTS;
    let permit = slots.permits.acquire().await.expect("semáforo aberto");
    let slot = slots
        .free
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .pop()
        .expect("um banco livre para cada permissão");
    let lease = Lease {
        slot,
        _permit: permit,
    };

    let options = slots.options[slot].clone();
    slots.ready[slot]
        .get_or_init(|| prepare(options.clone()))
        .await;
    reset(&options).await;
    let name = options.get_database().expect("nome do banco").to_owned();
    let id = NEXT_LEASE.fetch_add(1, Ordering::Relaxed);
    LEASES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(name, id);

    let pool = PgPoolOptions::new()
        .max_connections(CONNECTIONS)
        .connect_with(options)
        .await
        .expect("conexão com o banco de teste");
    (pool, lease)
}

/// O banco que um teste está usando; ao sair de escopo, volta para os outros.
pub struct Lease {
    slot: usize,
    _permit: SemaphorePermit<'static>,
}

impl Drop for Lease {
    fn drop(&mut self) {
        SLOTS
            .free
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(self.slot);
    }
}

/// O que separa um teste do outro no mesmo banco (o Redis e a caixa de emails
/// do teste usam): o nome do banco e de qual empréstimo dele é.
pub fn test_key(db: &PgPool) -> String {
    let options = db.connect_options();
    let name = options.get_database().unwrap_or("postgres");
    match LEASES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(name)
    {
        Some(lease) => format!("{name}:{lease}"),
        None => name.to_owned(),
    }
}

struct Slots {
    permits: Semaphore,
    free: Mutex<Vec<usize>>,
    options: Vec<PgConnectOptions>,
    /// Criado (ou conferido) e migrado nesta execução.
    ready: Vec<OnceCell<()>>,
}

static SLOTS: LazyLock<Slots> = LazyLock::new(|| {
    let count = std::env::var("TEST_DATABASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(DEFAULT_DATABASES);
    let main = main_options();
    // Com o prefixo `_sqlx_test_`, que o pg_hba.conf do README libera para o
    // usuário do projeto: `tournaments` → `_sqlx_test_tournaments_t0`, e o
    // banco de um worktree (`_sqlx_test_wt_x`) → `_sqlx_test_wt_x_t0`.
    let main_name = main.get_database().unwrap_or("postgres");
    let base = main_name.strip_prefix("_sqlx_test_").unwrap_or(main_name);
    Slots {
        permits: Semaphore::new(count),
        free: Mutex::new((0..count).rev().collect()),
        options: (0..count)
            .map(|i| main.clone().database(&format!("_sqlx_test_{base}_t{i}")))
            .collect(),
        ready: (0..count).map(|_| OnceCell::new()).collect(),
    }
});

static NEXT_LEASE: AtomicU64 = AtomicU64::new(1);
static LEASES: LazyLock<Mutex<HashMap<String, u64>>> = LazyLock::new(Default::default);

/// O SQL que esvazia um banco de teste, o mesmo para todos (o schema é igual).
static RESET: OnceLock<String> = OnceLock::new();

/// Uma suíte por vez na máquina: todos os worktrees usam o mesmo Postgres, e
/// duas suítes juntas se atrasam mais do que uma depois da outra. O lock vale
/// até o processo sair.
static SUITE_LOCK: LazyLock<File> = LazyLock::new(|| {
    let path = std::env::temp_dir().join("tournaments-api-tests.lock");
    let file = File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .expect("arquivo do lock dos testes");
    match file.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => {
            // Direto no stderr: o `eprintln!` dentro de um teste fica preso na
            // saída capturada dele.
            let _ = writeln!(
                std::io::stderr(),
                "outra suíte de testes está rodando nesta máquina; esperando ela terminar ({})",
                path.display()
            );
            file.lock().expect("lock dos testes");
        }
        Err(TryLockError::Error(err)) => panic!("lock dos testes: {err}"),
    }
    file
});

fn main_options() -> PgConnectOptions {
    std::env::var("DATABASE_URL")
        .expect("os testes precisam do DATABASE_URL")
        .parse()
        .expect("DATABASE_URL inválido")
}

/// Deixa o banco de teste pronto: o que já existe com as migrations de agora
/// fica; o que não existe, ou está com outras (outra branch, migration nova),
/// é criado de novo, migrado e ganha a cópia das linhas das migrations.
async fn prepare(options: PgConnectOptions) {
    let name = options.get_database().expect("nome do banco").to_owned();
    let mut main = PgConnection::connect_with(&main_options())
        .await
        .expect("os testes precisam do Postgres do DATABASE_URL");
    let exists: bool =
        sqlx::query_scalar("select exists (select 1 from pg_database where datname = $1)")
            .bind(&name)
            .fetch_one(&mut main)
            .await
            .unwrap();
    if exists {
        let mut conn = PgConnection::connect_with(&options).await.unwrap();
        let current = up_to_date(&mut conn).await;
        conn.close().await.ok();
        if current {
            return;
        }
        sqlx::raw_sql(AssertSqlSafe(format!(
            r#"drop database "{name}" with (force)"#
        )))
        .execute(&mut main)
        .await
        .unwrap();
    }
    sqlx::raw_sql(AssertSqlSafe(format!(r#"create database "{name}""#)))
        .execute(&mut main)
        .await
        .unwrap();
    main.close().await.ok();

    let pool = PgPool::connect_with(options).await.unwrap();
    api::MIGRATOR
        .run(&pool)
        .await
        .expect("migrations no banco de teste");
    let tables: Vec<String> = sqlx::query_scalar(
        "select tablename::text from pg_tables
         where schemaname = 'public' and tablename <> '_sqlx_migrations'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let mut seed = format!("create schema {SEED_SCHEMA};");
    for table in tables {
        let filled: bool = sqlx::query_scalar(AssertSqlSafe(format!(
            r#"select exists (select 1 from "{table}")"#
        )))
        .fetch_one(&pool)
        .await
        .unwrap();
        if filled {
            seed.push_str(&format!(
                r#"create table {SEED_SCHEMA}."{table}" as table public."{table}";"#
            ));
        }
    }
    sqlx::raw_sql(AssertSqlSafe(seed))
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;
}

/// As migrations aplicadas são as de agora, e a cópia das linhas delas existe.
async fn up_to_date(conn: &mut PgConnection) -> bool {
    let applied: Result<Vec<(i64, Vec<u8>)>, _> = sqlx::query_as(
        "select version, checksum from _sqlx_migrations where success order by version",
    )
    .fetch_all(&mut *conn)
    .await;
    let expected: Vec<(i64, Vec<u8>)> = api::MIGRATOR
        .iter()
        .filter(|m| !m.migration_type.is_down_migration())
        .map(|m| (m.version, m.checksum.to_vec()))
        .collect();
    let seeded: bool = sqlx::query_scalar("select to_regnamespace($1) is not null")
        .bind(SEED_SCHEMA)
        .fetch_one(&mut *conn)
        .await
        .unwrap_or(false);
    applied.is_ok_and(|applied| applied == expected) && seeded
}

/// Tira o que o teste anterior gravou: derruba as conexões que sobraram dele,
/// esvazia as tabelas (as sequências voltam ao 1) e põe de volta as linhas das
/// migrations, que um teste pode ter mudado.
async fn reset(options: &PgConnectOptions) {
    let mut conn = PgConnection::connect_with(options).await.unwrap();
    sqlx::raw_sql(
        "select pg_terminate_backend(pid) from pg_stat_activity
         where datname = current_database() and pid <> pg_backend_pid()
           and backend_type = 'client backend'",
    )
    .execute(&mut conn)
    .await
    .unwrap();
    if RESET.get().is_none() {
        let sql = reset_sql(&mut conn).await;
        let _ = RESET.set(sql);
    }
    sqlx::raw_sql(RESET.get().unwrap().as_str())
        .execute(&mut conn)
        .await
        .expect("esvaziar o banco de teste");
    conn.close().await.ok();
}

async fn reset_sql(conn: &mut PgConnection) -> String {
    let tables: Vec<(String, bool)> = sqlx::query_as(
        "select t.tablename::text, s.tablename is not null
         from pg_tables t
         left join pg_tables s on s.schemaname = $1 and s.tablename = t.tablename
         where t.schemaname = 'public' and t.tablename <> '_sqlx_migrations'
         order by 1",
    )
    .bind(SEED_SCHEMA)
    .fetch_all(&mut *conn)
    .await
    .unwrap();
    let empty: Vec<String> = tables
        .iter()
        .filter(|(_, seeded)| !seeded)
        .map(|(t, _)| format!(r#""{t}""#))
        .collect();
    let mut sql = format!("truncate table {} restart identity;", empty.join(", "));
    for (table, _) in tables.iter().filter(|(_, seeded)| *seeded) {
        sql.push_str(&format!(
            r#"delete from "{table}"; insert into "{table}" overriding system value select * from {SEED_SCHEMA}."{table}";"#
        ));
    }
    sql
}

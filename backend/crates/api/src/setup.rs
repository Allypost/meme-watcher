use config::CONFIG;
use migration::MigratorTrait;
use sea_orm::{ConnectOptions, DatabaseConnection, DbErr};
use sqlx::sqlite::SqliteConnectOptions;

pub(super) async fn setup_db() -> Result<DatabaseConnection, DbErr> {
    use sea_orm::Database;

    let database_url = CONFIG
        .database
        .url
        .clone()
        .or_else(|| {
            CONFIG
                .db_path()
                .to_str()
                .map(|x| format!("sqlite://{}?mode=rwc", x))
        })
        .unwrap_or_else(|| "sqlite::memory:".to_string());

    logger::debug!(url = ?database_url, "Connecting to database");

    let opts = ConnectOptions::new(database_url);
    let db = Database::connect(opts).await?;

    let sqlite_pool = db.get_sqlite_connection_pool();
    let mut sqlite_opts = SqliteConnectOptions::new();
    sqlite_opts.clone_from(&sqlite_pool.connect_options());

    sqlite_pool.set_connect_options(
        sqlite_opts
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .auto_vacuum(sqlx::sqlite::SqliteAutoVacuum::Incremental)
            .create_if_missing(true)
            .optimize_on_close(true, None),
    );

    migration::Migrator::up(&db, None).await?;

    Ok(db)
}

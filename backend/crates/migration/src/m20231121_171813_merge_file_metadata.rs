use sea_orm_migration::prelude::*;

use crate::CURRENT_TIMESTAMP;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        {
            let stmt = Table::alter()
                .table(Files::Table)
                .add_column(
                    ColumnDef::new(Files::FileType)
                        .string()
                        .extra("collate nocase"),
                )
                .to_owned();
            manager.alter_table(stmt).await?;

            let stmt = Table::alter()
                .table(Files::Table)
                .add_column(ColumnDef::new(Files::FileSize).big_integer())
                .to_owned();
            manager.alter_table(stmt).await?;

            let stmt = Table::alter()
                .table(Files::Table)
                .add_column(ColumnDef::new(Files::FileCtime).timestamp())
                .to_owned();
            manager.alter_table(stmt).await?;

            let stmt = Table::alter()
                .table(Files::Table)
                .add_column(ColumnDef::new(Files::FileMtime).timestamp())
                .to_owned();
            manager.alter_table(stmt).await?;

            manager
                .get_connection()
                .execute_unprepared(
                    r#"
                    UPDATE "files" SET
                        "file_type" = "file_metadata"."file_type",
                        "file_size" = "file_metadata"."file_size",
                        "file_ctime" = "file_metadata"."file_ctime",
                        "file_mtime" = "file_metadata"."file_mtime" 
                    FROM "file_metadata"
                    WHERE "files"."id" = "file_metadata"."file_id"
                    "#,
                )
                .await?;
        }

        {
            let stmt = Table::drop()
                .table(FileMetadata::Table)
                .if_exists()
                .to_owned();

            manager.drop_table(stmt).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        {
            let mut fk_file_id = ForeignKey::create()
                .from_tbl(FileMetadata::Table)
                .from_col(FileMetadata::FileId)
                .to_tbl(Files::Table)
                .to_col(Files::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .to_owned();

            let stmt = Table::create()
                .table(FileMetadata::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(FileMetadata::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(FileMetadata::FileId)
                        .integer()
                        .not_null()
                        .unique_key(),
                )
                .col(
                    ColumnDef::new(FileMetadata::FileType)
                        .string()
                        .extra("collate nocase"),
                )
                .col(ColumnDef::new(FileMetadata::FileSize).big_integer())
                .col(ColumnDef::new(FileMetadata::FileCtime).timestamp())
                .col(ColumnDef::new(FileMetadata::FileMtime).timestamp())
                .col(
                    ColumnDef::new(FileMetadata::CreatedAt)
                        .timestamp()
                        .default(CURRENT_TIMESTAMP.clone())
                        .not_null(),
                )
                .foreign_key(&mut fk_file_id)
                .to_owned();

            manager.create_table(stmt).await?;

            let stmt = Index::create()
                .if_not_exists()
                .name(format!(
                    "{}__idx__{}",
                    FileMetadata::Table.to_string(),
                    FileMetadata::FileId.to_string(),
                ))
                .table(FileMetadata::Table)
                .col(FileMetadata::FileMtime)
                .to_owned();

            manager.create_index(stmt).await?;

            let stmt = Index::create()
                .if_not_exists()
                .name(format!(
                    "{}__idx__{}",
                    FileMetadata::Table.to_string(),
                    FileMetadata::FileType.to_string(),
                ))
                .table(FileMetadata::Table)
                .col(FileMetadata::FileType)
                .to_owned();

            manager.create_index(stmt).await?;

            manager
                .get_connection()
                .execute_unprepared(
                    r#"
                    INSERT INTO "file_metadata"
                        ("file_id", "file_type", "file_size", "file_ctime", "file_mtime", "created_at")
                    SELECT
                        "id", "file_type", "file_size", "file_ctime", "file_mtime", "created_at"
                    FROM "files"
                    "#,
                )
                .await?;
        }

        {
            let stmt = Table::alter()
                .table(Files::Table)
                .drop_column(Files::FileType)
                .to_owned();
            manager.alter_table(stmt).await?;

            let stmt = Table::alter()
                .table(Files::Table)
                .drop_column(Files::FileSize)
                .to_owned();
            manager.alter_table(stmt).await?;

            let stmt = Table::alter()
                .table(Files::Table)
                .drop_column(Files::FileCtime)
                .to_owned();
            manager.alter_table(stmt).await?;

            let stmt = Table::alter()
                .table(Files::Table)
                .drop_column(Files::FileMtime)
                .to_owned();
            manager.alter_table(stmt).await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Files {
    Table,
    Id,
    FileType,
    FileSize,
    FileCtime,
    FileMtime,
}

#[derive(DeriveIden)]
enum FileMetadata {
    Table,
    Id,
    FileId,
    FileType,
    FileSize,
    FileCtime,
    FileMtime,
    CreatedAt,
}

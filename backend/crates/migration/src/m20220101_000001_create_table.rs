use sea_orm_migration::prelude::*;

use crate::CURRENT_TIMESTAMP;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        {
            let stmt = Table::create()
                .table(Files::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Files::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(Files::Ulid)
                        .string()
                        .not_null()
                        .unique_key()
                        .extra("collate nocase"),
                )
                .col(ColumnDef::new(Files::Path).string().not_null().unique_key())
                .col(
                    ColumnDef::new(Files::Hash)
                        .string()
                        .not_null()
                        .extra("collate nocase"),
                )
                .col(
                    ColumnDef::new(Files::CreatedAt)
                        .timestamp()
                        .default(CURRENT_TIMESTAMP.clone())
                        .not_null(),
                )
                .to_owned();

            manager.create_table(stmt).await?;

            let stmt = Index::create()
                .if_not_exists()
                .name(format!(
                    "{}__idx__{}",
                    Files::Table.to_string(),
                    Files::Hash.to_string(),
                ))
                .table(Files::Table)
                .col(Files::Hash)
                .to_owned();

            manager.create_index(stmt).await?;
        }

        {
            let stmt = Table::create()
                .table(Tags::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Tags::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(Tags::Name).string().not_null())
                .col(
                    ColumnDef::new(Tags::CreatedAt)
                        .timestamp()
                        .default(CURRENT_TIMESTAMP.clone())
                        .not_null(),
                )
                .to_owned();

            manager.create_table(stmt).await?;
        }

        {
            let mut fk_file_id = ForeignKey::create()
                .from_tbl(FilesTags::Table)
                .from_col(FilesTags::FileId)
                .to_tbl(Files::Table)
                .to_col(Files::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .to_owned();

            let mut fk_tag_id = ForeignKey::create()
                .from_tbl(FilesTags::Table)
                .from_col(FilesTags::TagId)
                .to_tbl(Tags::Table)
                .to_col(Tags::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .to_owned();

            let stmt = Table::create()
                .table(FilesTags::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(FilesTags::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(FilesTags::FileId).integer().not_null())
                .col(ColumnDef::new(FilesTags::TagId).integer().not_null())
                .col(
                    ColumnDef::new(FilesTags::CreatedAt)
                        .timestamp()
                        .default(CURRENT_TIMESTAMP.clone())
                        .not_null(),
                )
                .foreign_key(&mut fk_file_id)
                .foreign_key(&mut fk_tag_id)
                .to_owned();

            manager.create_table(stmt).await?;
        }

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
        }

        {
            let mut fk_file_id = ForeignKey::create()
                .from_tbl(FileData::Table)
                .from_col(FileData::FileId)
                .to_tbl(Files::Table)
                .to_col(Files::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .to_owned();

            let stmt = Table::create()
                .if_not_exists()
                .table(FileData::Table)
                .col(
                    ColumnDef::new(FileData::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(FileData::FileId).integer().not_null())
                .col(
                    ColumnDef::new(FileData::Key)
                        .string()
                        .not_null()
                        .extra("collate nocase"),
                )
                .col(ColumnDef::new(FileData::Value).string().not_null())
                .col(
                    ColumnDef::new(FileData::Meta)
                        .json_binary()
                        .not_null()
                        .default("{}"),
                )
                .col(
                    ColumnDef::new(FileData::CreatedAt)
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
                    FileData::Table.to_string(),
                    FileData::FileId.to_string(),
                ))
                .table(FileData::Table)
                .col(FileData::FileId)
                .to_owned();

            manager.create_index(stmt).await?;

            let stmt = Index::create()
                .if_not_exists()
                .name(format!(
                    "{}__idx__{}",
                    FileData::Table.to_string(),
                    FileData::Key.to_string(),
                ))
                .table(FileData::Table)
                .col(FileData::Key)
                .to_owned();

            manager.create_index(stmt).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().if_exists().table(Files::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().if_exists().table(Tags::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().if_exists().table(FilesTags::Table).to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(FileMetadata::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().if_exists().table(FileData::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Files {
    Table,
    Id,
    Ulid,
    Path,
    Hash,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Tags {
    Table,
    Id,
    Name,
    CreatedAt,
}

#[derive(DeriveIden)]
enum FilesTags {
    Table,
    Id,
    FileId,
    TagId,
    CreatedAt,
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

#[derive(DeriveIden)]
enum FileData {
    Table,
    Id,
    FileId,
    Key,
    Value,
    Meta,
    CreatedAt,
}

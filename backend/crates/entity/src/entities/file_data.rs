//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.4

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "file_data")]
#[serde(rename_all = "camelCase")]
#[typeshare::typeshare]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub file_id: i32,
    pub key: String,
    pub value: String,
    pub meta: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::files::Entity",
        from = "Column::FileId",
        to = "super::files::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Files,
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Files.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
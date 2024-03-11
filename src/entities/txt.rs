//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "txt")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub title: String,
    #[sea_orm(unique)]
    pub hash: String,
    pub user_id: u64,
    pub level: u8,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

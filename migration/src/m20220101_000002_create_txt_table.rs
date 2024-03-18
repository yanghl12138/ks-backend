use sea_orm_migration::prelude::*;
use super::m20220101_000001_create_user_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(Txt::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Txt::Id)
                            .big_unsigned()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Txt::Title).string_len(30).not_null())
                    .col(ColumnDef::new(Txt::Hash).string_len(64).unique_key().not_null())
                    .col(ColumnDef::new(Txt::UserId).big_unsigned().not_null())
                    .col(ColumnDef::new(Txt::Level).tiny_unsigned().not_null().default(0))
                    .foreign_key(
                        ForeignKey::create()
                        .name("fk-txt-user-id")
                        .from(Txt::Table, Txt::UserId)
                        .to(User::Table, User::Id)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Txt::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Txt {
    Table,
    Id,
    Title,
    Hash,
    UserId,
    Level
}

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_user_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .auto_increment()
                            .big_unsigned()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(User::Username)
                            .string_len(12)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(User::IsAdmin).boolean().not_null())
                    .col(ColumnDef::new(User::Level).tiny_unsigned().not_null().default(0u8))
                    .col(ColumnDef::new(User::Password).string_len(64).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Username,
    IsAdmin,
    Level,
    Password
}

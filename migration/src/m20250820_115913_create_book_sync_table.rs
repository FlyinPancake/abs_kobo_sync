use crate::m20250820_115221_create_devices_table::Devices;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BookSync::Table)
                    .if_not_exists()
                    .col(uuid(BookSync::Id).primary_key())
                    .col(uuid(BookSync::DeviceId))
                    .col(string(BookSync::AbsItemId))
                    .col(timestamp(BookSync::Timestamp))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_book_sync_device_id")
                            .from(BookSync::Table, BookSync::DeviceId)
                            .to(Devices::Table, Devices::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BookSync::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum BookSync {
    Table,
    Id,
    DeviceId,
    AbsItemId,
    Timestamp,
}

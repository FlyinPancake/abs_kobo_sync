pub use sea_orm_migration::prelude::*;

mod m20250819_215543_create_user_table;
mod m20250820_115221_create_devices_table;
mod m20250820_115913_create_book_sync_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250819_215543_create_user_table::Migration),
            Box::new(m20250820_115221_create_devices_table::Migration),
            Box::new(m20250820_115913_create_book_sync_table::Migration),
        ]
    }
}

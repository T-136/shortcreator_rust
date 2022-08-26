use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        // todo!();

        manager
            .create_table(
                Table::create()
                    .table(Clip::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Clip::ClipId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Clip::GMapsLink).string())
                    .col(ColumnDef::new(Clip::Name).string())
                    .col(ColumnDef::new(Clip::Group).string())
                    .col(ColumnDef::new(Clip::Videotext).string())
                    .col(ColumnDef::new(Clip::Latlong).string())
                    .col(ColumnDef::new(Clip::Start).integer())
                    .col(ColumnDef::new(Clip::Stop).integer())
                    .col(ColumnDef::new(Clip::StreetviewVideo).string())
                    .col(ColumnDef::new(Clip::IsRenderd).string())
                    .col(ColumnDef::new(Clip::IsUploadedYt).string())
                    .col(ColumnDef::new(Clip::IsUploadedTikTok).string())
                    .col(ColumnDef::new(Clip::IsUploadedInstagram).string())
                    .col(ColumnDef::new(Clip::YmusicId).string())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Ymusic::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Ymusic::YmusicId)
                    .string()
                    .not_null()
                    .primary_key())
                    .col(ColumnDef::new(Ymusic::YmusicTitle).string())
                    .to_owned()
            ).await

        
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(Clip::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Ymusic::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden

#[derive(Iden)]
enum Clip {
    Table,
    ClipId,
    GMapsLink,
    Name,
    Group,
    Videotext,
    Latlong,
    Start,
    Stop,
    StreetviewVideo,
    IsRenderd,
    IsUploadedYt,
    IsUploadedTikTok,
    IsUploadedInstagram,
    YmusicId,
}

#[derive(Iden)]
enum Ymusic {
    Table,
    YmusicId,
    YmusicTitle,
}
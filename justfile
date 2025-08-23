default:
    just --choose

DATABASE_URL := "sqlite://db.sqlite?mode=rwc"

sea *ARGS: (_sea DATABASE_URL ARGS)
_sea $DATABASE_URL *ARGS: 
    sea-orm-cli {{ ARGS }} 

# Generate database entities
generate_entities: (_sea DATABASE_URL "generate entity --with-serde=both --output-dir=entities/src --lib")

migrate *ARGS: (_sea DATABASE_URL "migrate" ARGS)

run:
    cargo run

alias r := run
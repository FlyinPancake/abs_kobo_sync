default:
    just --choose

DATABASE_URL := "sqlite://db.sqlite"

sea *ARGS: (_sea DATABASE_URL ARGS)
_sea $DATABASE_URL *ARGS: 
    sea {{ ARGS }} 

# Generate database entities
generate_entities: (_sea DATABASE_URL "generate entity --with-serde=both --output-dir=entities/src --lib")

migrate: (_sea DATABASE_URL "migrate")
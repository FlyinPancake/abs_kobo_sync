default:
    just --choose

DATABASE_URL := "sqlite://db.sqlite"

_sea $DATABASE_URL *ARGS: 
    sea {{ ARGS }} 

# Generate database entities
generate_entities: (_sea DATABASE_URL "generate entity --with-serde=both --output-dir=entities/src --lib")

sea *ARGS: (_sea DATABASE_URL ARGS)
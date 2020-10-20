-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id uuid DEFAULT uuid_generate_v4() PRIMARY KEY,
    username VARCHAR NOT NULL UNIQUE,
    email VARCHAR NOT NULL UNIQUE,
    password_hash VARCHAR NOT NULL,
    full_name VARCHAR NULL,
    bio VARCHAR NULL,
    image VARCHAR NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE user_locations (
    id uuid DEFAULT uuid_generate_v4() PRIMARY KEY,
    street VARCHAR(30) NULL,
    city VARCHAR(30) NULL,
    state VARCHAR(30) NULL,
    country VARCHAR(30) NULL,
    user_id uuid REFERENCES users (id)
);

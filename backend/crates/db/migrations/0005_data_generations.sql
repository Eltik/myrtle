CREATE TABLE data_generations (
    server_id   smallint    PRIMARY KEY REFERENCES servers (id),
    generation  bigint      NOT NULL DEFAULT 0,
    res_version text,
    updated_at  timestamptz NOT NULL DEFAULT now()
);

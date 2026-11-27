CREATE TABLE service_account_sessions (
    server_id  smallint    PRIMARY KEY REFERENCES servers (id),
    nonce      bytea       NOT NULL,
    ciphertext bytea       NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);

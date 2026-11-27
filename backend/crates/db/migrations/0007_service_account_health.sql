ALTER TABLE service_account_sessions
    ADD COLUMN refused_since timestamptz,
    ADD COLUMN last_ok_at    timestamptz;

ALTER TABLE data_generations
    ADD COLUMN pipeline_seq bigint NOT NULL DEFAULT 0,
    ADD COLUMN recorded_seq bigint NOT NULL DEFAULT 0;

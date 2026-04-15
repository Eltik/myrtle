CREATE TABLE operator_notes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operator_id     VARCHAR(50) NOT NULL UNIQUE,
    pros            TEXT,
    cons            TEXT,
    notes           TEXT,
    trivia          TEXT,
    summary         VARCHAR(500),
    tags            JSONB DEFAULT '[]'::JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE operator_notes_audit_log (
    id              BIGSERIAL PRIMARY KEY,
    note_id         UUID NOT NULL REFERENCES operator_notes(id) ON DELETE CASCADE,
    field_name      VARCHAR(50) NOT NULL,
    old_value       TEXT,
    new_value       TEXT,
    changed_by      UUID NOT NULL REFERENCES users(id),
    changed_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_operator_notes_audit_note_id ON operator_notes_audit_log(note_id);

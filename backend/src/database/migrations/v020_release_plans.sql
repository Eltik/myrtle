CREATE TABLE IF NOT EXISTS release_plans (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    initial integer NOT NULL DEFAULT 0,
    initial_manual boolean NOT NULL DEFAULT false,
    picks jsonb NOT NULL DEFAULT '[]'::jsonb,
    stages jsonb NOT NULL DEFAULT '{}'::jsonb,
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- Create solver submissions table
CREATE TABLE solver_submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    solver_name VARCHAR(255) NOT NULL,
    wasm_hash VARCHAR(64) NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_solver_submissions_user_id ON solver_submissions(user_id);
CREATE INDEX idx_solver_submissions_wasm_hash ON solver_submissions(wasm_hash);
CREATE INDEX idx_solver_submissions_submitted_at ON solver_submissions(submitted_at DESC);

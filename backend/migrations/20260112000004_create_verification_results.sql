-- Create verification results table
CREATE TABLE verification_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id UUID NOT NULL REFERENCES solver_submissions(id) ON DELETE CASCADE,
    map_name VARCHAR(255) NOT NULL,
    scenario_id VARCHAR(255) NOT NULL,
    num_agents INTEGER NOT NULL,
    valid BOOLEAN NOT NULL,
    cost BIGINT,
    makespan BIGINT,
    instruction_count BIGINT,
    execution_time_ms BIGINT NOT NULL,
    error_message TEXT,
    verified_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_verification_results_submission_id ON verification_results(submission_id);
CREATE INDEX idx_verification_results_map_name ON verification_results(map_name);
CREATE INDEX idx_verification_results_valid ON verification_results(valid);
CREATE INDEX idx_verification_results_leaderboard ON verification_results(valid, cost, instruction_count) WHERE valid = true;

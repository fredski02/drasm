-- Module registry table
CREATE TABLE modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    storage_path TEXT NOT NULL UNIQUE,
    hash TEXT NOT NULL UNIQUE,
    size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Job metadata table
CREATE TABLE jobs (
    job_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    module_id UUID REFERENCES modules(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    input_message JSONB NOT NULL,
    result JSONB,
    error TEXT,
    worker_id TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    
    CONSTRAINT valid_status CHECK (status IN ('pending', 'processing', 'completed', 'failed'))
);

-- Indexes for performance
CREATE INDEX idx_jobs_user_id ON jobs(user_id);
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_created_at ON jobs(created_at DESC);
CREATE INDEX idx_modules_user_id ON modules(user_id);
CREATE INDEX idx_modules_hash ON modules(hash);

-- Enable Row Level Security
ALTER TABLE modules ENABLE ROW LEVEL SECURITY;
ALTER TABLE jobs ENABLE ROW LEVEL SECURITY;

-- RLS Policies for modules
CREATE POLICY "Users can view own modules" ON modules
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own modules" ON modules
    FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update own modules" ON modules
    FOR UPDATE USING (auth.uid() = user_id);

CREATE POLICY "Users can delete own modules" ON modules
    FOR DELETE USING (auth.uid() = user_id);

-- RLS Policies for jobs
CREATE POLICY "Users can view own jobs" ON jobs
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own jobs" ON jobs
    FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update own jobs" ON jobs
    FOR UPDATE USING (auth.uid() = user_id);

CREATE POLICY "Service role can update any job" ON jobs
    FOR UPDATE USING (auth.role() = 'service_role');

-- Storage bucket for WASM modules
INSERT INTO storage.buckets (id, name, public)
VALUES ('wasm-modules', 'wasm-modules', false);

-- Storage policies for wasm-modules bucket
CREATE POLICY "Users can upload own modules"
ON storage.objects FOR INSERT
WITH CHECK (
    bucket_id = 'wasm-modules' 
    AND auth.uid()::text = (storage.foldername(name))[1]
);

CREATE POLICY "Users can read own modules"
ON storage.objects FOR SELECT
USING (
    bucket_id = 'wasm-modules'
    AND auth.uid()::text = (storage.foldername(name))[1]
);

CREATE POLICY "Service role can read all modules"
ON storage.objects FOR SELECT
USING (
    bucket_id = 'wasm-modules'
    AND auth.role() = 'service_role'
);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
CREATE TRIGGER update_modules_updated_at
    BEFORE UPDATE ON modules
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

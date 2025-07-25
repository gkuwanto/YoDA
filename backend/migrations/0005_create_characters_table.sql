-- Create characters table
CREATE TABLE characters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    player_id UUID REFERENCES users(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    race VARCHAR(100),
    class VARCHAR(100),
    level INTEGER DEFAULT 1,
    hp_current INTEGER,
    hp_max INTEGER,
    ac INTEGER,
    speed INTEGER,
    stats JSONB DEFAULT '{}',
    inventory JSONB DEFAULT '[]',
    spells JSONB DEFAULT '[]',
    features JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for characters
CREATE INDEX idx_characters_campaign_id ON characters(campaign_id);
CREATE INDEX idx_characters_player_id ON characters(player_id);
CREATE INDEX idx_characters_name ON characters(name);

-- Add game_state column to sessions if it doesn't exist (it should already exist)
-- This is just to ensure the column exists for the enhanced game state
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS game_state JSONB DEFAULT '{}';

-- Create a function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
CREATE TRIGGER update_characters_updated_at BEFORE UPDATE ON characters
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column(); 
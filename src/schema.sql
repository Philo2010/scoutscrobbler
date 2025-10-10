CREATE TABLE scouting_entry (
    id SERIAL PRIMARY KEY,
    "user" TEXT,
    team INTEGER,
    matchid INTEGER,
    total_score INTEGER,
    event_code TEXT,
    tournament_level VARCHAR(20),
    station VARCHAR(6),
    is_verified VARCHAR(20),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE auto_data (
    id SERIAL PRIMARY KEY,
    scouting_id INTEGER REFERENCES scouting_entry(id) ON DELETE CASCADE,
    moved BOOLEAN,
    L1 INTEGER,
    L2 INTEGER,
    L3 INTEGER,
    L4 INTEGER,
    algae_processor INTEGER,
    algae_barge INTEGER,
    algae_remove INTEGER
);

CREATE TABLE teleop_data (
    id SERIAL PRIMARY KEY,
    scouting_id INTEGER REFERENCES scouting_entry(id) ON DELETE CASCADE,
    L1 INTEGER,
    L2 INTEGER,
    L3 INTEGER,
    L4 INTEGER,
    algae_processor INTEGER,
    algae_barge INTEGER,
    algae_remove INTEGER
);

CREATE TABLE endgame_data (
    id SERIAL PRIMARY KEY,
    died BOOLEAN,
    scouting_id INTEGER REFERENCES scouting_entry(id) ON DELETE CASCADE,
    defense_rating INTEGER,
    climb_type TEXT,
    comment TEXT
);

CREATE TABLE user_list (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    can_write BOOLEAN NOT NULL,
    can_read BOOLEAN NOT NULL,
    is_admin BOOLEAN NOT NULL
);

CREATE TABLE matches (
    id SERIAL PRIMARY KEY,
    event_code TEXT NOT NULL,
    match_number INTEGER NOT NULL,
    description TEXT NOT NULL,
    tournament_level TEXT NOT NULL
);

CREATE TABLE match_teams (
    id SERIAL PRIMARY KEY,
    match_id INTEGER NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    team_number INTEGER NOT NULL,
    station TEXT NOT NULL
);

CREATE TABLE pit_data (
    id SERIAL PRIMARY KEY,
    team INTEGER NOT NULL,
    event_code TEXT NOT NULL,
    comment TEXT NOT NULL
);
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
    algae_processor BOOLEAN NOT NULL,
    algae_barge BOOLEAN NOT NULL,
    algae_remove BOOLEAN NOT NULL,
    L1 BOOLEAN NOT NULL,
    L2 BOOLEAN NOT NULL,
    L3 BOOLEAN NOT NULL,
    L4 BOOLEAN NOT NULL,
    auto_align BOOLEAN NOT NULL,
    ground_intake BOOLEAN NOT NULL,
    climber BOOLEAN NOT NULL,
    height TEXT NOT NULL,
    widthxlength TEXT NOT NULL,
    weight TEXT NOT NULL,
    defence BOOLEAN NOT NULL,
    driver_years_experience TEXT NOT NULL,
    comment TEXT NOT NULL,
    UNIQUE (team, event_code)
);


CREATE TABLE pit_auto_data (
    id SERIAL PRIMARY KEY,
    pit_id INTEGER REFERENCES pit_data(id) ON DELETE CASCADE,
    --Auto
    left_auto BOOLEAN NOT NULL,
    center_auto BOOLEAN NOT NULL,
    right_auto BOOLEAN NOT NULL,

    --Comment
    amount_of_sides TEXT NOT NULL,
    amount_of_combo_sides TEXT NOT NULL,
    coral_amount TEXT NOT NULL,
    algae_amount TEXT NOT NULL
);
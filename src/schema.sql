PRAGMA foreign_keys = ON;

CREATE TABLE scouting_entry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user TEXT,
    team INTEGER,
    matchid INTEGER,
    total_score INTEGER,
    event_code TEXT,
    tournament_level VARCHAR(20),
    station VARCHAR(6),
    is_verified VARCHAR(20),
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE auto_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scouting_id INTEGER,
    moved BOOL,
    L1 INTEGER,
    L2 INTEGER,
    L3 INTEGER,
    L4 INTEGER,
    algae_processor INTEGER,
    algae_barge INTEGER,
    algae_remove INTEGER,
    FOREIGN KEY(scouting_id) REFERENCES scouting_entry(id)
);

CREATE TABLE teleop_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scouting_id INTEGER,
    L1 INTEGER,
    L2 INTEGER,
    L3 INTEGER,
    L4 INTEGER,
    algae_processor INTEGER,
    algae_barge INTEGER,
    algae_remove INTEGER,
    FOREIGN KEY(scouting_id) REFERENCES scouting_entry(id)
);

CREATE TABLE endgame_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    died BOOL,
    scouting_id INTEGER,
    defense_rating INTEGER,
    climb_type TEXT,
    comment TEXT,
    FOREIGN KEY(scouting_id) REFERENCES scouting_entry(id)
);


CREATE TABLE user_list (
    id BLOB PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    can_write BOOL NOT NULL,
    can_read BOOL NOT NULL,
    is_admin BOOL NOT NULL
);

CREATE TABLE matches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_code TEXT NOT NULL,
    match_number INTEGER NOT NULL,
    description TEXT NOT NULL,
    tournament_level TEXT NOT NULL
);

CREATE TABLE match_teams (
    match_id INTEGER NOT NULL,
    team_number INTEGER NOT NULL,
    station TEXT NOT NULL,
    PRIMARY KEY (match_id, station),
    FOREIGN KEY (match_id) REFERENCES matches(id) ON DELETE CASCADE
);
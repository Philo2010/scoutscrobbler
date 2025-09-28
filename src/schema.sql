CREATE TABLE scouting_entry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user TEXT,
    team INTEGER,
    matchid INTEGER,
    total_score INTEGER,
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
    can_read BOOL NOT NULL
);
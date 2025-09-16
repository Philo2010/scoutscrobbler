CREATE TABLE scouting_entry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE auto_data (
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
    scouting_id INTEGER,
    defense_rating INTEGER,
    climb_type TEXT,
    comment TEXT,
    FOREIGN KEY(scouting_id) REFERENCES scouting_entry(id)
);


CREATE TABLE user_list (
    id BLOB PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    passhash TEXT NOT NULL,
    can_write BOOL NOT NULL,
    can_read BOOL NOT NULL
);
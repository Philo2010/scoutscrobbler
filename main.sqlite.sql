BEGIN TRANSACTION;
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS "auto_data" (
	"id"	INTEGER,
	"scouting_id"	INTEGER,
	"moved"	BOOL,
	"L1"	INTEGER,
	"L2"	INTEGER,
	"L3"	INTEGER,
	"L4"	INTEGER,
	"algae_processor"	INTEGER,
	"algae_barge"	INTEGER,
	"algae_remove"	INTEGER,
	PRIMARY KEY("id" AUTOINCREMENT),
	FOREIGN KEY("scouting_id") REFERENCES "scouting_entry"("id")
);
CREATE TABLE IF NOT EXISTS "endgame_data" (
	"id"	INTEGER,
	"died"	BOOL,
	"scouting_id"	INTEGER,
	"defense_rating"	INTEGER,
	"climb_type"	TEXT,
	"comment"	TEXT,
	PRIMARY KEY("id" AUTOINCREMENT),
	FOREIGN KEY("scouting_id") REFERENCES "scouting_entry"("id")
);
CREATE TABLE IF NOT EXISTS "events" (
	"id"	INTEGER,
	"event_code"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("id" AUTOINCREMENT)
);
CREATE TABLE IF NOT EXISTS "match_teams" (
	"match_id"	INTEGER NOT NULL,
	"team_id"	INTEGER NOT NULL,
	"station"	TEXT NOT NULL,
	PRIMARY KEY("match_id","station"),
	FOREIGN KEY("match_id") REFERENCES "matches"("id") ON DELETE CASCADE,
	FOREIGN KEY("team_id") REFERENCES "teams"("id") ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS "matches" (
	"id"	INTEGER,
	"event_id"	INTEGER NOT NULL,
	"match_number"	INTEGER NOT NULL,
	"description"	TEXT NOT NULL,
	"tournament_level"	TEXT NOT NULL,
	PRIMARY KEY("id" AUTOINCREMENT),
	FOREIGN KEY("event_id") REFERENCES "events"("id") ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS "scouting_entry" (
	"id"	INTEGER,
	"user"	TEXT,
	"team"	INTEGER,
	"matchid"	INTEGER,
	"total_score"	INTEGER,
	"event_code"	TEXT,
	"tournament_level"	VARCHAR(20),
	"station"	VARCHAR(6),
	"is_verified"	VARCHAR(20),
	"created_at"	TEXT DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY("id" AUTOINCREMENT)
);
CREATE TABLE IF NOT EXISTS "teams" (
	"id"	INTEGER,
	"team_number"	INTEGER NOT NULL UNIQUE,
	PRIMARY KEY("id" AUTOINCREMENT)
);
CREATE TABLE IF NOT EXISTS "teleop_data" (
	"id"	INTEGER,
	"scouting_id"	INTEGER,
	"L1"	INTEGER,
	"L2"	INTEGER,
	"L3"	INTEGER,
	"L4"	INTEGER,
	"algae_processor"	INTEGER,
	"algae_barge"	INTEGER,
	"algae_remove"	INTEGER,
	PRIMARY KEY("id" AUTOINCREMENT),
	FOREIGN KEY("scouting_id") REFERENCES "scouting_entry"("id")
);
CREATE TABLE IF NOT EXISTS "user_list" (
	"id"	BLOB,
	"username"	TEXT NOT NULL UNIQUE,
	"can_write"	BOOL NOT NULL,
	"can_read"	BOOL NOT NULL,
	"is_admin"	BOOL NOT NULL,
	PRIMARY KEY("id")
);
INSERT INTO "user_list" VALUES (X'd0c1b0ed2a0642b9bc6d5588436258a9','philo',1,1,1);
COMMIT;

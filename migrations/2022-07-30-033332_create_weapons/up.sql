-- Your SQL goes here
CREATE TABLE IF NOT EXISTS weapons (
	id SERIAL PRIMARY KEY,
	name VARCHAR NOT NULL,
	icon CHAR NOT NULL,
	damage_range INT4RANGE DEFAULT '[10, 20]',
	crit_ratio FLOAT8 DEFAULT 2.0 / 100.0,
	crit_multiplier INT DEFAULT 2,
	pierce INT DEFAULT 0
)
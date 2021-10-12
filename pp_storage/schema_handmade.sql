-- SELECT session_user AS sess_user, current_database() AS curr_db;

DROP TABLE IF EXISTS works;
CREATE TABLE IF NOT EXISTS works (
	id         SERIAL PRIMARY KEY,
	work_code  VARCHAR ( 50 ) NOT NULL,
	add_up_to  INT NOT NULL, -- rust type i32
	done       BOOLEAN DEFAULT FALSE,
	updated_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);


DROP TABLE IF EXISTS events;
CREATE TABLE IF NOT EXISTS events (
	id         SERIAL PRIMARY KEY,
	work_code  VARCHAR ( 50 ) NOT NULL,  -- TODO perhaps foreign key to `works` table
	variable   VARCHAR ( 100 ) NOT NULL, -- TODO figure out this domain of variables/operations
	value      VARCHAR ( 100 ) NOT NULL, -- we parse the string to int/float/bool later in Rust code
	created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

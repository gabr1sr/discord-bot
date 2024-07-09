-- Add migration script here
CREATE TABLE tags (
       id      BIGSERIAL NOT NULL PRIMARY KEY,
       name    TEXT 	 NOT NULL UNIQUE,
       content TEXT   	 NOT NULL,
       owner   TEXT   	 NOT NULL
)

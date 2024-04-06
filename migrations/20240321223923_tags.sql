-- Add migration script here
CREATE TABLE tags (
       id    	  serial	NOT NULL PRIMARY KEY,
       user_id	  TEXT		NOT NULL,
       name	  TEXT		NOT NULL,
       content	  TEXT		NOT NULL
);

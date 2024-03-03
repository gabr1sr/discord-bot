CREATE TABLE user_infractions (
       id    	     integer	 NOT NULL PRIMARY KEY,
       user_id	     TEXT    	 NOT NULL,
       infraction_id integer	 NOT NULL,
       created_at    TIMESTAMPTZ DEFAULT Now()
);

CREATE TYPE severity AS ENUM ('low', 'mid', 'high');

CREATE TYPE punishment AS ENUM ('strike', 'timeout', 'ban', 'kick');

CREATE TABLE infractions (
       id    	  integer    NOT NULL PRIMARY KEY,
       severity	  severity   NOT NULL,
       punishment punishment NOT NULL,
       duration	  bigint     NOT NULL DEFAULT 0
);

CREATE TABLE punishments (
       id    	  serial      PRIMARY KEY,
       user_id	  TEXT	      NOT NULL,
       punishment punishment  NOT NULL,
       duration	  bigint      NOT NULL DEFAULT 0
);

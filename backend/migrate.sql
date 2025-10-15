-- ALWAYS BACKUP FIRST

-- The script was create because sqlite do not support alter foreign key
-- 1. Rename db.sqlite* to backup.sqlite*(Please ensure WAL is also renamed)
-- 2. Run this script on duckdb

LOAD sqlite;

ATTACH 'backup.sqlite' (TYPE sqlite);
ATTACH 'db.sqlite' (TYPE sqlite);

-- Parents first
INSERT INTO db.main.user SELECT * FROM backup.main.user;
INSERT INTO db.main.model SELECT * FROM backup.main.model;
INSERT INTO db.main.config SELECT * FROM backup.main.config;

-- Mid-level
INSERT INTO db.main.chat SELECT * FROM backup.main.chat;

-- Children
INSERT INTO db.main.message SELECT * FROM backup.main.message;
INSERT INTO db.main.file SELECT * FROM backup.main.file;
INSERT INTO db.main.tool SELECT * FROM backup.main.tool;
INSERT INTO db.main.chunk SELECT * FROM backup.main.chunk;

-- System
INSERT INTO db.main.seaql_migrations SELECT * FROM backup.main.seaql_migrations;

-- The script was create because sqlite do not support alter foreign key
-- Run this script on libsql with PRAGMA writable_schema=on
-- ALWAYS BACKUP FIRST

ALTER TABLE chat DROP CONSTRAINT 'fk-chat-model_id-model';

ALTER TABLE chat ALTER COLUMN model_id DROP NOT NULL;

ALTER TABLE chat
ADD CONSTRAINT 'fk-chat-model_id-model'
FOREIGN KEY (model_id) REFERENCES model(id)
ON UPDATE CASCADE ON DELETE SET NULL;

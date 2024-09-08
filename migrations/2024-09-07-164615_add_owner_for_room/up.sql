-- Your SQL goes here
ALTER TABLE rooms ADD COLUMN owner_id TEXT NOT NULL DEFAULT "default_user" REFERENCES users(id);

-- select one user from rooms_users table and set it as owner
UPDATE rooms SET owner_id = (SELECT user_id FROM rooms_users LIMIT 1);

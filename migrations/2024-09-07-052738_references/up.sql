-- Your SQL goes here
CREATE TABLE rooms (
  id TEXT PRIMARY KEY NOT NULL,
  name VARCHAR NOT NULL,
  last_message TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE users (
  id TEXT PRIMARY KEY NOT NULL,
  username VARCHAR NOT NULL,
  password VARCHAR NOT NULL,
  created_at TEXT NOT NULL,
  unique(username)
);

CREATE TABLE conversations (
  id TEXT PRIMARY KEY NOT NULL,
  room_id TEXT NOT NULL REFERENCES rooms(id),
  user_id TEXT NOT NULL REFERENCES users(id),
  message TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE rooms_users (
  room_id TEXT REFERENCES rooms(id),
  user_id TEXT REFERENCES users(id),
  PRIMARY KEY (room_id, user_id)
);

-- Your SQL goes here
CREATE TABLE new_rooms_users (
    room_id TEXT NOT NULL REFERENCES rooms(id),
    user_id TEXT NOT NULL REFERENCES users(id),
    PRIMARY KEY (room_id, user_id)
);

INSERT INTO new_rooms_users SELECT * FROM rooms_users;

DROP TABLE rooms_users;

ALTER TABLE new_rooms_users RENAME TO rooms_users;

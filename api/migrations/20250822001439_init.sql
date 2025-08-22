CREATE TABLE users (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    username TEXT,
    email BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE events (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    uri TEXT NOT NULL,
    language TEXT NOT NULL,
    line_number INT NOT NULL,
    cursor_pos INT NOT NULL,
    user_id INT REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL
);

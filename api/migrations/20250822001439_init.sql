CREATE TABLE users (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE events (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    uri TEXT NOT NULL,
    is_write BOOL NOT NULL,
    language TEXT,
    line_number INT,
    cursor_pos INT,
    user_id INT NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE auth_tokens (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    token TEXT UNIQUE NOT NULL,
    user_id INT NOT NULL REFERENCES users(id),
    disabled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL
);

ALTER TABLE users
ADD CONSTRAINT uq_users_email UNIQUE (email);

ALTER TABLE users
ALTER COLUMN email TYPE TEXT;

CREATE TABLE auth_tokens (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id INT REFERENCES users(id),
    token BYTEA,
    active BOOL,
    created_at TIMESTAMPTZ
);

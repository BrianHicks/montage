CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL CHECK(kind IN ('task', 'break')),
    description TEXT NOT NULL,
    start_time DATETIME NOT NULL,
    duration STRING NOT NULL,
    end_time DATETIME
);

CREATE INDEX idx_start_time ON sessions (start_time);

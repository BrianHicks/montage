CREATE TABLE sessions (
    ID INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT CHECK(kind IN ('task', 'break')),
    description TEXT,
    start_time DATETIME NOT NULL,
    duration STRING NOT NULL,
    end_time DATETIME
);

CREATE INDEX idx_start_time ON sessions (start_time);

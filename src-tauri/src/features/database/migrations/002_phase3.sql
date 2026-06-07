CREATE TABLE IF NOT EXISTS sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  app_name TEXT NOT NULL,
  window_title TEXT,
  started_at INTEGER NOT NULL,
  ended_at INTEGER,
  duration_secs INTEGER
);

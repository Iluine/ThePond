-- ─── Event configuration (single row) ───────────────────────────
CREATE TABLE IF NOT EXISTS event_config (
    id INTEGER PRIMARY KEY,
    event_name TEXT NOT NULL,
    expected_guests_count INTEGER NOT NULL,
    theme_path TEXT NOT NULL,
    witness_token_hash TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- ─── Users (canards) ─────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    pseudo TEXT NOT NULL,
    custom_name TEXT,
    duck_color TEXT NOT NULL CHECK (duck_color IN ('yellow', 'white', 'blue', 'rainbow')),
    created_at TEXT NOT NULL,
    last_seen_at TEXT
);

-- ─── Phases (paliers) ────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS phases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL,
    phase_order INTEGER NOT NULL,
    name TEXT NOT NULL,
    target_time TEXT NOT NULL,
    triggered_at TEXT,
    is_final_reveal INTEGER NOT NULL DEFAULT 0,
    UNIQUE (event_id, phase_order)
);

CREATE INDEX IF NOT EXISTS idx_phases_target ON phases (event_id, target_time);

-- ─── Photos (barbotages) ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS media (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (id),
    filename TEXT NOT NULL,
    thumb_filename TEXT NOT NULL,
    posted_at TEXT NOT NULL,
    hidden INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_media_posted_at ON media (posted_at);

-- ─── Clips (cancans) ─────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS clips (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (id),
    filename TEXT NOT NULL,
    thumb_filename TEXT NOT NULL,
    duration_seconds REAL NOT NULL,
    posted_at TEXT NOT NULL,
    hidden INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_clips_posted_at ON clips (posted_at);

-- ─── Voice messages (coin-coin) ──────────────────────────────────
CREATE TABLE IF NOT EXISTS voice_messages (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (id),
    filename TEXT NOT NULL,
    waveform_json TEXT NOT NULL,
    duration_seconds REAL NOT NULL,
    caption TEXT,
    posted_at TEXT NOT NULL,
    hidden INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_voice_posted_at ON voice_messages (posted_at);

-- ─── Likes (cœurs) ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS likes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL REFERENCES users (id),
    content_type TEXT NOT NULL CHECK (content_type IN ('media', 'clip', 'voice')),
    content_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (user_id, content_type, content_id)
);

CREATE INDEX IF NOT EXISTS idx_likes_content ON likes (content_type, content_id);

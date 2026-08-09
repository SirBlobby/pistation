CREATE TABLE IF NOT EXISTS admins (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS kiosks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    location TEXT NOT NULL DEFAULT '',
    room_name TEXT NOT NULL UNIQUE,
    hardware_id TEXT,
    enrollment_token_hash TEXT,
    kiosk_token_hash TEXT,
    status TEXT NOT NULL DEFAULT 'offline',
    last_seen_at INTEGER,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS kiosk_pins (
    pin TEXT PRIMARY KEY,
    kiosk_id TEXT NOT NULL REFERENCES kiosks(id) ON DELETE CASCADE,
    issued_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_kiosk_pins_kiosk ON kiosk_pins(kiosk_id, issued_at DESC);
CREATE INDEX IF NOT EXISTS idx_kiosk_pins_expiry ON kiosk_pins(expires_at);

CREATE TABLE IF NOT EXISTS kiosk_layouts (
    kiosk_id TEXT PRIMARY KEY REFERENCES kiosks(id) ON DELETE CASCADE,
    data TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    kiosk_id TEXT NOT NULL REFERENCES kiosks(id) ON DELETE CASCADE,
    participant_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    role TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    revoked INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_sessions_kiosk ON sessions(kiosk_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expiry ON sessions(expires_at);

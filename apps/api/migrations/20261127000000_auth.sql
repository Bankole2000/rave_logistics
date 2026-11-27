-- Sign-in with a phone number and a one-time code, and revocable sessions (R-30, ADR-008).

-- One row per person, whatever they use the product as. Customer, driver, vendor and staff
-- details live in their own tables and point here, so one phone number is one identity.
CREATE TABLE users (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    -- E.164, e.g. +2348031234567. Verified: a user is only created after a code is entered.
    phone        text NOT NULL UNIQUE CHECK (phone ~ '^\+[1-9][0-9]{7,14}$'),
    created_at   timestamptz NOT NULL DEFAULT now(),
    -- Set when the account is suspended (FT-073); suspended users can't sign in or use sessions.
    suspended_at timestamptz
);

-- One-time codes. Only a hash of the code is stored.
CREATE TABLE sign_in_codes (
    id         uuid PRIMARY KEY,
    phone      text NOT NULL,
    code_hash  bytea NOT NULL,
    attempts   integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL,
    used_at    timestamptz
);
-- Rate limits look up the recent codes of one phone number.
CREATE INDEX sign_in_codes_phone_created_at ON sign_in_codes (phone, created_at DESC);

-- A signed-in device (R-30). The client holds a random token; only its SHA-256 hash is stored,
-- so a leaked database can't be used to sign in.
CREATE TABLE sessions (
    id           uuid PRIMARY KEY,
    user_id      uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token_hash   bytea NOT NULL UNIQUE,
    -- Shown in a future "signed-in devices" list; supplied by the client, e.g. "Pixel 8".
    device_name  text CHECK (char_length(device_name) <= 100),
    created_at   timestamptz NOT NULL DEFAULT now(),
    last_seen_at timestamptz NOT NULL DEFAULT now(),
    revoked_at   timestamptz
);
CREATE INDEX sessions_active_by_user ON sessions (user_id) WHERE revoked_at IS NULL;

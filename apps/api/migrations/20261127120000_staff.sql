-- Staff roles and invitations (FT-072, FT-073) and sign-in by email code (FT-098, FT-099).

-- Staff and vendor owners may sign in by email only, so a user has a phone number, an email
-- address, or both. Both are verified: they are only stored after a code sent to them is entered.
ALTER TABLE users ALTER COLUMN phone DROP NOT NULL;
ALTER TABLE users ADD COLUMN email text UNIQUE
    CHECK (email = lower(email) AND email LIKE '_%@_%.__%');
ALTER TABLE users ADD CONSTRAINT users_phone_or_email CHECK (phone IS NOT NULL OR email IS NOT NULL);

-- A code goes to a phone number (sms) or an email address (email).
ALTER TABLE sign_in_codes RENAME COLUMN phone TO destination;
ALTER TABLE sign_in_codes ADD COLUMN channel text NOT NULL DEFAULT 'sms'
    CHECK (channel IN ('sms', 'email'));
ALTER TABLE sign_in_codes ALTER COLUMN channel DROP DEFAULT;
ALTER INDEX sign_in_codes_phone_created_at RENAME TO sign_in_codes_destination_created_at;

-- Internal staff (R-22). One role each; what a role may do is defined in src/auth/roles.rs.
CREATE TABLE staff_members (
    user_id      uuid PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    role         text NOT NULL
        CHECK (role IN ('support', 'operations', 'admin', 'hr', 'finance', 'management')),
    -- NULL for the first admin, created with the invite-admin command.
    invited_by   uuid REFERENCES users (id) ON DELETE SET NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    -- A suspended staff member is signed out everywhere and can't sign in (FT-073).
    suspended_at timestamptz
);

-- An invitation to join as staff, accepted by signing in with the invited phone number or email.
CREATE TABLE staff_invitations (
    id          uuid PRIMARY KEY,
    phone       text,
    email       text,
    role        text NOT NULL
        CHECK (role IN ('support', 'operations', 'admin', 'hr', 'finance', 'management')),
    invited_by  uuid REFERENCES users (id) ON DELETE SET NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    expires_at  timestamptz NOT NULL,
    accepted_at timestamptz,
    accepted_by uuid REFERENCES users (id) ON DELETE SET NULL,
    CHECK ((phone IS NULL) <> (email IS NULL))
);
CREATE INDEX staff_invitations_phone ON staff_invitations (phone) WHERE accepted_at IS NULL;
CREATE INDEX staff_invitations_email ON staff_invitations (email) WHERE accepted_at IS NULL;

CREATE TYPE reservation_status AS ENUM ('unknown', 'pending', 'confirmed', 'blocked');
CREATE TYPE reservation_event AS ENUM ('unknown', 'create', 'update', 'delete');

CREATE TABLE reservations (
    id BIGSERIAL NOT NULL,
    user_id VARCHAR(64) NOT NULL,
    status reservation_status NOT NULL DEFAULT 'pending',
    resource_id VARCHAR(64) NOT NULL,
    timespan TSTZRANGE NOT NULL,
    note TEXT,
    CONSTRAINT reservations_pk PRIMARY KEY (id),
    CONSTRAINT reservations_conflict EXCLUDE USING gist (resource_id WITH =, timespan WITH &&)
);
CREATE INDEX reservations_resource_id_idx ON reservations (resource_id);
CREATE INDEX reservations_user_id_idx ON reservations (user_id);

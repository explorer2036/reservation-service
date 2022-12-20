
-- resevation event queue
CREATE TABLE reservation_events (
    id SERIAL NOT NULL,
    reservation_id BIGSERIAL NOT NULL,
    old JSONB,
    new JSONB,
    event reservation_event NOT NULL,
    CONSTRAINT reservation_events_pkey PRIMARY KEY (id)
);
CREATE INDEX reservation_events_reservation_id_event_idx ON reservation_events (reservation_id, event);

-- server read cursor
CREATE TABLE server_read_cursor (
    server_id VARCHAR(64) NOT NULL,
    last_change_id BIGSERIAL NOT NULL,
    CONSTRAINT reservation_events_cursor_pkey PRIMARY KEY (server_id)
);

-- trigger for add/update/delete a reservation
CREATE OR REPLACE FUNCTION reservations_trigger() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- update reservation_events
        INSERT INTO reservation_events (reservation_id, old, new, event) VALUES (NEW.id, null, to_jsonb(NEW), 'create');
    ELSIF TG_OP = 'UPDATE' THEN
        -- if status changed, update reservation_events
        IF OLD.status <> NEW.status THEN
            INSERT INTO reservation_events (reservation_id, old, new, event) VALUES (NEW.id, to_jsonb(OLD), to_jsonb(NEW), 'update');
        END IF;
    ELSIF TG_OP = 'DELETE' THEN
        -- update reservation_events
        INSERT INTO reservation_events (reservation_id, old, new, event) VALUES (OLD.id, to_jsonb(OLD), null, 'delete');
    END IF;
    -- notify a channel called reservation_event
    NOTIFY reservation_event;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reservations_trigger
    AFTER INSERT OR UPDATE OR DELETE ON reservations
    FOR EACH ROW EXECUTE PROCEDURE reservations_trigger();

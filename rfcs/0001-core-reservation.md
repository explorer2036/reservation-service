# Core Reservation Service

- Feature Name: core-reservation-service
- Start Date: 2022-12-19 18:49:30

## Summary

A core reservation service that solves the problem of reserving a resource for a specific time period. We leverage postgres EXCLUDE constraints to ensure that only one reservation can exist at a time for a given resource.

## Motivation

We need a common solution for various reservation requirements: 1) calendar booking; 2) hotel/room booking; 3) meeting room booking; 4) parking lot booking; 5) etc. Repeatedly implementing the same solution in different services is not only a waste of time, but also a waste of resources. We need a common solution that can be used by all services.

## Guide-level explanation

### Basic Architecture

![architecture_draft](./architecture_draft.png)

### Service interface

We would use gRPC as a service interface. Below is the proto definition:

```proto
syntax = "proto3";
package reservation;

import "google/protobuf/timestamp.proto";

// reservation status for a given time period
enum ReservationStatus {
    RESERVATION_STATUS_UNKNOWN = 0;
    RESERVATION_STATUS_PENDING = 1;
    RESERVATION_STATUS_CONFIRMED = 2;
    RESERVATION_STATUS_BLOCKED = 3;
}

// when reservation is updated, record the reservation event type
enum ReservationEvent {
    RESERVATION_EVENT_UNKNOWN = 0;
    RESERVATION_EVENT_CREATED = 1;
    RESERVATION_EVENT_UPDATED = 2;
    RESERVATION_EVENT_DELETED = 3;
}

// Core reservation object. Contains all the information for a reservation
// if ListenResponse event is DELETE, only id will be populated
message Reservation {
    // unique id for the reservation, if put into ReserveRequest, id should be empty
    string id = 1;
    // user id for the reservation
    string user_id = 2;
    // reservation status, used for different purpose
    ReservationStatus status = 3;
    // resource id for the reservation
    string resource_id = 4;
    // start time for the reservation
    google.protobuf.Timestamp start = 5;
    // end time for the reservation
    google.protobuf.Timestamp end = 6;
    // extra note
    string note = 7;
}

// To make a reservation, send a ReserveRequest with Reservation object (id should be empty)
message ReserveRequest {
    Reservation reservation = 1;
}

// Created reservation will be returned in ReserveResponse
message ReserveResponse {
    Reservation reservation = 1;
}

// To update a reservation, send an UpdateRequest. Only note is updatable.
message UpdateRequest {
    string note = 1;
}

// Updated reservation will be returned in UpdateResponse
message UpdateResponse {
    Reservation reservation = 1;
}

// To change a reservation from pending to confirmed, send a ConfirmRequest
message ConfirmRequest {
    string id = 1;
}

// Confirmed reservation will be returned in ConfirmResponse
message ConfirmResponse {
    Reservation reservation = 1;
}

// To cancel a reservation, send a CancelRequest
message CancelRequest {
    string id = 1;
}

// Canceled reservation will be returned in CancelResponse
message CancelResponse {
    Reservation reservation = 1;
}

// To get a reservation, send a GetRequest
message GetRequest {
    string id = 1;
}

// Reservation will be returned in GetResponse
message GetResponse {
    Reservation reservation = 1;
}

// query reservations with user id, resource id, start time, end time, and status
message ReservationQuery {
    // resource id for the reservation query. If empty, query all resources
    string resource_id = 1;
    // user id for the reservation query. If empty, query all users
    string user_id = 2;
    // use status to filter result. If UNKNOWN, return all reservations
    ReservationStatus status = 3;
    // start time for the reservation query, if 0, use Infinity for start time
    google.protobuf.Timestamp start = 4;
    // end time for the reservation query, if 0, use Infinity for end time
    google.protobuf.Timestamp end = 5 ;
    // sort direction
    bool desc = 6;
}

// To query reservations, send a QueryRequest
message QueryRequest {
    ReservationQuery query = 1;
}

// query reservations, order by reservation id
message ReservationFilter {
    // resource id for the reservation query. If empty, query all resources
    string resource_id = 1;
    // user id for the reservation query. If empty, query all users
    string user_id = 2;
    // use status to filter result. If UNKNOWN, return all reservations
    ReservationStatus status = 3;
    int64 cursor = 4;
    // page size for the query
    int64 page_size = 5;
    // sort direction
    bool desc = 6;
}

// To filter reservations, send a FilterRequest
message FilterRequest {
    ReservationFilter filter = 1;
}

// filter pager info
message FilterPager {
    int64 prev = 1;
    int64 next = 2;
    int64 total = 3;
}

message FilterResponse {
    repeated Reservation reservations = 1;
    FilterPager pager = 2;
}

// Client can listen to reservation events by sending a ListenRequest
message ListenRequest {
}

// Server will send ListenResponse to client in streaming response
message ListenResponse {
    // event type
    ReservationEvent event = 1;
    // id for updated reservation
    Reservation reservation = 2;
}

// Reservation service
service ReservationService {
    // make a reservation
    rpc reserve(ReserveRequest) returns (ReserveResponse);
    // confirm a pending reservation, if reservation is not pending, do nothing
    rpc confirm(ConfirmRequest) returns (ConfirmResponse);
    // update the reservation note
    rpc update(UpdateRequest) returns (UpdateResponse);
    // cancel a reservation
    rpc cancel(CancelRequest) returns (CancelResponse);
    // get a reservation by id
    rpc get(GetRequest) returns (GetResponse);
    // query reservations by resource id, user id, status, start time, end time
    rpc query(QueryRequest) returns (stream Reservation);
    // filter reservations, order by reservation id
    rpc filter(FilterRequest) returns (FilterResponse);
    // another system could monitor reservation events: added/confirmed/cancelled
    rpc listen(ListenRequest) returns (stream Reservation);
}
```

### Database schema

We use postgres as the database. Below is the schema:

```sql
CREATE TYPE reservation_status AS ENUM ('unknown', 'pending', 'confirmed', 'blocked');
CREATE TYPE reservation_event AS ENUM ('unknown', 'create', 'update', 'delete');

CREATE TABLE reservation (
    id uuid NOT NULL DEFAULT uuid_generate_v4(),
    user_id varchar(64) NOT NULL,
    status reservation_status NOT NULL DEFAULT 'pending',
    resource_id varchar(64) NOT NULL,
    span tstzrange NOT NULL,
    note text,
    CONSTRAINT reservation_pk PRIMARY KEY (id),
    CONSTRAINT reservation_conflict EXCLUDE USING gist (resource_id WITH =, span WITH &&)
);
CREATE INDEX reservation_resource_id_idx ON reservation (resource_id);
CREATE INDEX reservation_user_id_idx ON reservation (user_id);

-- if user_id is null, find all reservations within duration for the resource
-- if resource_id is null, find all reservations within duration for the user
-- if both are null, find all reservations within duration
-- if both set, find all reservations within duration for the resource and user
CREATE OR REPLACE FUNCTION query(uid text, rid text, duration tstzrange) RETURNS TABLE reservation AS $$ $$ LANGUAGE plpgsql;

-- reservation event queue
CREATE TABLE reservation_events (
    id SERIAL NOT NULL,
    reservation_id uuid NOT NULL,
    event reservation_event NOT NULL
);

-- trigger for add/update/delete a reservation
CREATE OR REPLACE FUNCTION reservations_trigger() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- update reservation_events
        INSERT INTO reservation_events (reservation_id, event) VALUES (NEW.id, 'create');
    ELSIF TG_OP = 'UPDATE' THEN
        -- if status changed, update reservation_events
        IF OLD.status <> NEW.status THEN
            INSERT INTO reservation_events (reservation_id, event) VALUES (NEW.id, 'update');
        END IF;
    ELSIF TG_OP = 'DELETE' THEN
        -- update reservation_events
        INSERT INTO reservation_events (reservation_id, event) VALUES (OLD.id, 'delete');
    END IF;
    -- notify a channel called reservation_event
    NOTIFY reservation_event;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reservations_trigger
    AFTER INSERT OR UPDATE OR DELETE ON reservations
    FOR EACH ROW EXECUTE PROCEDURE reservations_trigger();
```

## Reference-level explanation

TBD

## Drawbacks

N/A

## Rationale and alternatives

N/A

## Prior art

N/A

## Unresolved questions

- how to handle repeated reservation? - is this more or less a business logic which shouldn't be put into this layer? (non-goal: we consider this is a business logic and should be handled by the caller)
- if load is big, we may use an external queue for recording changes.
- we haven't considered tracking/observability/deployment yet.
- query performance might be an issue - need to revisit the index and also consider using cache.

## Future possibilities

N/A

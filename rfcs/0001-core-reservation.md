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

```text
sudo vmhgfs-fuse .host:/ /mnt/hgfs/ -o allow_other -o uid=1000 -o gid=1000 -o umask=0002
```

### Service interface

We would use gRPC as a service interface. Below is the proto definition:

```proto
enum ReservationStatus {
    UNKNOWN = 0;
    PENDING = 1;
    CONFIRMED = 2;
    BLOCKED = 3;
}

message Reservation {
    string id = 1;
    string user_id = 2;
    ReservationStatus status = 3;

    // resource reservation window
    string resource_id = 4;
    google.protobuf.Timestamp start_time = 5;
    google.protobuf.Timestamp end_time = 6;

    // extra note
    string note = 7;
}

message ReserveRequest {
    Reservation reservation = 1;
}

message ReserveResponse {
    Reservation reservation = 1;
}

message UpdateRequest {
    string note = 1;
}

message UpdateResponse {
    Reservation reservation = 1;
}

message ConfirmRequest {
    string id = 1;
}

message ConfirmResponse {
    Reservation reservation = 1;
}

message CancelRequest {
    string id = 1;
}

message CancelResponse {
    Reservation reservation = 1;
}

message GetRequest {
    string id = 1;
}

message GetResponse {
    Reservation reservation = 1;
}

message QueryRequest {
    string resource_id = 1;
    string user_id = 2;
    ReservationStatus status = 3;
    google.protobuf.Timestamp start_time = 4;
    google.protobuf.Timestamp end_time = 5 ;
}

message WatchRequest {
}

service ReservationService {
    rpc reserve(ReserveRequest) returns (ReserveResponse);
    rpc confirm(ConfirmRequest) returns (ConfirmResponse);
    rpc update(UpdateRequest) returns (UpdateResponse);
    rpc cancel(CancelRequest) returns (CancelResponse);
    rpc get(GetRequest) returns (GetResponse);
    rpc query(QueryRequest) returns (stream Reservation);
    // another system could monitor reservation events
    rpc watch(WatchRequest) returns (stream Reservation);
}
```

### Database schema

We use postgres as the database. Below is the schema:

```sql
CREATE TYPE reservation_status AS ENUM ('unknown', 'pending', 'confirmed', 'blocked');

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

CREATE OR REPLACE FUNCTION query(uid text, rid text, duration tstzrange) RETURNS TABLE reservation AS $$ $$ LANGUAGE plpgsql;
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

N/A

## Future possibilities

N/A


---/------------------------\---
--|          TYPES          |--
---\------------------------/---

-- status of a trip
CREATE TYPE trip_status AS ENUM(
    'scheduled',
    'unscheduled',
    'cancelled',
    'added',
    'deleted' -- see gtfs rt
);

---/------------------------\---
--|          TABLES          |--
---\------------------------/---

CREATE TABLE trip_updates(
    origin              slug NOT NULL REFERENCES origins(id),
    trip_id             slug NOT NULL,
    trip_start_date     DATE NOT NULL,
    status              trip_status,
    stop_time_updates   JSONB,
    timestamp           TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY(origin, trip_id, trip_start_date)
);

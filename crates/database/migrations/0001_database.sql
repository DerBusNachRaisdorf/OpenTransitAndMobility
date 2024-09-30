---/------------------------\---
--|         SETTINGS        |--
---\------------------------/---

ALTER DATABASE public_transport SET timezone TO 'Europe/Berlin';

---/------------------------\---
--|        EXTENSIONS       |--
---\------------------------/---

-- for better text search
CREATE EXTENSION IF NOT EXISTS pg_trgm;
SET pg_trgm.similarity_threshold = 0.2; -- default: 0.3

---/------------------------\---
--|          TYPES          |--
---\------------------------/---

CREATE DOMAIN slug AS TEXT
    CONSTRAINT slug_is_lowercase_alphanumeric
    CHECK (VALUE ~ '\A[a-z0-9-]*\Z')
    CONSTRAINT slug_contains_no_repeating_dashes
    CHECK (VALUE !~ '-{2,}');

CREATE TYPE service_availability AS ENUM('available', 'unavailable');

CREATE TYPE service_exception_type AS ENUM('added', 'removed');

CREATE TYPE line_type as ENUM(
    'tram_streetcar_or_lightrail',
    'subway_or_metro',
    'rail',
    'bus',
    'ferry',
    'cable_tram',
    'aerial_lift_or_suspended_cable_car',
    'funicular',
    'trolleybus',
    'monorail'
);

---/------------------------\---
--|        SEQUENCES         |--
---\------------------------/---

-- service id sequence, used by 'calendar_windows' und 'calendar_dates'
CREATE SEQUENCE service_id_seq;

-- shapes
CREATE SEQUENCE shape_id_seq;

---/------------------------\---
--|          TABLES          |--
---\------------------------/---

-- origins
CREATE TABLE origins(
    id              slug NOT NULL,
    name            TEXT NOT NULL,
    priority        INT NOT NULL,
    PRIMARY KEY(id)
);

-- collectors
CREATE TABLE collectors(
    id              INTEGER GENERATED ALWAYS AS IDENTITY,
    origin          slug NOT NULL REFERENCES origins(id),
    kind            TEXT NOT NULL,
    is_active       BOOL NOT NULL,
    state           JSONB NOT NULL
);

-- agencies
CREATE TABLE agencies(
    id              slug NOT NULL,
    origin          slug NOT NULL REFERENCES origins(id),
    name            TEXT NOT NULL,
    website         TEXT NOT NULL,
    phone_number    TEXT,
    email           TEXT,
    fare_url        TEXT,
    PRIMARY KEY(id, origin)
);

CREATE INDEX ON agencies ((lower(name)));

CREATE TABLE agencies_original_ids(
    origin          slug NOT NULL REFERENCES origins(id),
    original_id     TEXT NOT NULL,
    id              slug NOT NULL,
    PRIMARY KEY(original_id, origin),
    FOREIGN KEY(id, origin) REFERENCES agencies(id, origin)
);

CREATE INDEX ON agencies_original_ids(id, origin);

-- stops
CREATE TABLE stops(
    id              slug NOT NULL,
    origin          slug NOT NULL REFERENCES origins(id),
    name            TEXT,
    description     TEXT,
    parent_id       slug,
    latitude        DOUBLE PRECISION,
    longitude       DOUBLE PRECISION,
    address         TEXT,
    platform_code   TEXT,
    PRIMARY KEY(id, origin),
    FOREIGN KEY(parent_id, origin) REFERENCES stops(id, origin)
);

CREATE INDEX idx_stop_name_trgm ON stops USING GIN (name gin_trgm_ops);

CREATE TABLE stops_original_ids(
    origin          slug NOT NULL REFERENCES origins(id),
    original_id     TEXT NOT NULL,
    id              slug NOT NULL,
    PRIMARY KEY(original_id, origin),
    FOREIGN KEY(id, origin) REFERENCES stops(id, origin)
);

CREATE INDEX ON stops_original_ids(id, origin);

-- calendar
-- like gtfs calendar.txt, but different from gtfs, multiple calendars can exist with
-- the same id, but different start-/ end_dates.
CREATE TABLE calendar_windows(
    service_id      INT DEFAULT nextval('service_id_seq'),
    monday          service_availability NOT NULL,
    tuesday         service_availability NOT NULL,
    wednesday       service_availability NOT NULL,
    thursday        service_availability NOT NULL,
    friday          service_availability NOT NULL,
    saturday        service_availability NOT NULL,
    sunday          service_availability NOT NULL,
    start_date      DATE NOT NULL,
    end_date        DATE NOT NULL,
    PRIMARY KEY(service_id, start_date, end_date)
);

CREATE INDEX ON calendar_windows(service_id);

-- calendar dates
CREATE TABLE calendar_dates(
    service_id      INT DEFAULT nextval('service_id_seq'),
    date            DATE NOT NULL,
    exception_type  service_exception_type NOT NULL,
    PRIMARY KEY(service_id, date)
);

CREATE TABLE services_original_ids(
    origin          slug NOT NULL REFERENCES origins(id),
    original_id     TEXT NOT NULL,
    id              INT NOT NULL,
    PRIMARY KEY(original_id, origin)
);

CREATE INDEX ON services_original_ids(id, origin);

-- lines (called 'routes' in gtfs)
CREATE TABLE lines(
    id              slug NOT NULL,
    origin          slug NOT NULL REFERENCES origins(id),
    name            TEXT,
    kind            line_type NOT NULL,
    agency_id       slug,
    PRIMARY KEY(id, origin),
    FOREIGN KEY(agency_id, origin) REFERENCES agencies(id, origin)
);

CREATE INDEX ON lines ((lower(name)));

CREATE TABLE lines_original_ids(
    origin          slug NOT NULL REFERENCES origins(id),
    original_id     TEXT NOT NULL,
    id              slug NOT NULL,
    PRIMARY KEY(original_id, origin),
    FOREIGN KEY(id, origin) REFERENCES lines(id, origin)
);

CREATE INDEX ON lines_original_ids(id, origin);

-- shapes
CREATE TABLE shapes(
    id              INT DEFAULT nextval('shape_id_seq'),
    sequence        INT NOT NULL,
    latitude        DOUBLE PRECISION NOT NULL,
    longitude       DOUBLE PRECISION NOT NULL,
    distance        DOUBLE PRECISION,
    PRIMARY KEY(id, sequence)
);

CREATE INDEX ON shapes(id);

-- trips
CREATE TABLE trips(
    id              slug NOT NULL,
    origin          slug NOT NULL REFERENCES origins(id),
    line_id         slug NOT NULL,
    service_id      INT,
    headsign        TEXT,
    short_name      TEXT,
    --shape_id        INT REFERENCES shapes(id),
    -- todo: other columns
    PRIMARY KEY(id, origin),
    FOREIGN KEY(line_id, origin) REFERENCES lines(id, origin)
);

CREATE TABLE trips_original_ids(
    origin          slug NOT NULL REFERENCES origins(id),
    original_id     TEXT NOT NULL,
    id              slug NOT NULL,
    PRIMARY KEY(original_id, origin),
    FOREIGN KEY(id, origin) REFERENCES trips(id, origin)
);

CREATE INDEX ON trips_original_ids(id, origin);

-- stop_times
-- TODO: continuous pickup drop off / pick up!
CREATE TABLE stop_times(
    origin          slug NOT NULL REFERENCES origins(id),
    trip_id         slug NOT NULL,
    stop_sequence   INTEGER NOT NULL,
    stop_id         slug,
    arrival_time    BIGINT, -- time in seconds, to allow more than 24 hours.
    departure_time  BIGINT, -- time in seconds, to allow more than 24 hours.
    stop_headsign   TEXT, -- overrides trips.headsign if set
    PRIMARY KEY(origin, trip_id, stop_sequence),
    FOREIGN KEY(trip_id, origin) REFERENCES trips(id, origin),
    FOREIGN KEY(stop_id, origin) REFERENCES stops(id, origin)
);

CREATE INDEX on stop_times(stop_id);

-- vehicles
-- TODO: use this table lul
-- TODO: vehicle kind, e.g., train, bus, etc.
CREATE TABLE vehicles(
    id              slug NOT NULL,
    origin          slug NOT NULL REFERENCES origins(id),
    trip_id         slug,
    latitude        DOUBLE PRECISION,
    longitude       DOUBLE PRECISION,
    FOREIGN KEY(trip_id, origin) REFERENCES trips(id, origin)
);

---/-------------------------\---
--|       ID GENERATION       |--
---\-------------------------/---

-- creates a slug from any text input
CREATE OR REPLACE FUNCTION create_slug(VARIADIC columns TEXT[])
RETURNS slug AS $$
DECLARE
    result TEXT;
BEGIN
    -- concat all inputs with dashes
    result := lower(array_to_string(columns, '-'));
    -- replace all spaces with dashes
    result := regexp_replace(result, '\s', '-', 'g');
    -- remove all non-alphanumeric characters
    result := regexp_replace(result, '[^a-z0-9-]', '', 'g');
    -- replace repeated consecutive dashes with one dash
    result := regexp_replace(result, '-{2,}', '-', 'g');
    -- remove leading and trailing dashes
    result := trim(both '-' from result);

    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- origins
CREATE OR REPLACE FUNCTION set_origin_id()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.id IS NULL THEN
        NEW.id := create_slug(NEW.name);
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER before_insert_generate_origin_id
BEFORE INSERT ON origins
FOR EACH ROW
EXECUTE FUNCTION set_origin_id();

-- agencies
CREATE OR REPLACE FUNCTION set_agency_id()
RETURNS TRIGGER AS $$
DECLARE
    base_id TEXT;
    counter INTEGER := 1;
    final_id TEXT;
BEGIN
    IF NEW.id IS NULL THEN
        base_id := create_slug(NEW.name);
        final_id := base_id;

        WHILE EXISTS (SELECT 1 FROM agencies WHERE id = final_id) LOOP
            counter := counter + 1;
            final_id := concat(base_id, '-', counter);
        END LOOP;

        NEW.id := final_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER before_insert_generate_agency_id
BEFORE INSERT ON agencies
FOR EACH ROW
EXECUTE FUNCTION set_agency_id();

-- stops
CREATE OR REPLACE FUNCTION set_stop_id()
RETURNS TRIGGER AS $$
DECLARE
    base_id TEXT;
    counter INTEGER := 1;
    final_id TEXT;
BEGIN
    IF NEW.id IS NULL THEN
        base_id := create_slug(NEW.name, NEW.platform_code);
        final_id := base_id;

        WHILE EXISTS (SELECT 1 FROM stops WHERE id = final_id) LOOP
            counter := counter + 1;
            final_id := concat(base_id, '-', counter);
        END LOOP;

        NEW.id := final_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER before_insert_generate_stop_id
BEFORE INSERT ON stops
FOR EACH ROW
EXECUTE FUNCTION set_stop_id();

-- lines
CREATE OR REPLACE FUNCTION set_line_id()
RETURNS TRIGGER AS $$
DECLARE
    base_id TEXT;
    counter INTEGER := 1;
    final_id TEXT;
BEGIN
    IF NEW.id IS NULL THEN
        base_id := create_slug(NEW.agency_id, NEW.name);
        final_id := base_id;

        WHILE EXISTS (SELECT 1 FROM lines WHERE id = final_id) LOOP
            counter := counter + 1;
            final_id := concat(base_id, '-', counter);
        END LOOP;

        NEW.id := final_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER before_insert_generate_line_id
BEFORE INSERT ON lines
FOR EACH ROW
EXECUTE FUNCTION set_line_id();

-- trips
CREATE OR REPLACE FUNCTION set_trip_id()
RETURNS TRIGGER AS $$
DECLARE
    base_id TEXT;
    counter INTEGER := 1;
    final_id TEXT;
BEGIN
    IF NEW.id IS NULL THEN
        base_id := create_slug(NEW.line_id, NEW.headsign, NEW.service_id::TEXT);
        final_id := base_id;

        WHILE EXISTS (SELECT 1 FROM trips WHERE id = final_id AND origin = NEW.origin) LOOP
            counter := counter + 1;
            final_id := concat(base_id, '-', counter);
        END LOOP;

        NEW.id := final_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER before_insert_generate_trip_id
BEFORE INSERT ON trips
FOR EACH ROW
EXECUTE FUNCTION set_trip_id();

---/-------------------------\---
--|        EXAMPLE DATA       |--
---\-------------------------/---

--INSERT INTO agencies(origin, name, website)
--VALUES ('migration', 'bahn sh', 'https://bahn.sh');
--INSERT INTO agencies(origin, name, website)
--VALUES ('migration', 'DB Regio', 'https://bahn.de');
--INSERT INTO agencies(origin, id, name, website)
--VALUES ('migration', 'der-metronom', 'Metronom', 'https://metronom.de');

--INSERT INTO lines(origin, short_name, kind, agency_id)
--VALUES ('migration', 'RE83', 'rail', 'erixx');

--INSERT INTO lines(origin, short_name, kind, agency_id)
--VALUES ('migration', 'RE83', 'rail', 'erixx');

--INSERT INTO lines(origin, short_name, kind, agency_id)
--VALUES ('migration', 'RE83', 'rail', 'bahn-sh');

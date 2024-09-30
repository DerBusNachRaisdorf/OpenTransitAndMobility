---/------------------------\---
--|          TABLES          |--
---\------------------------/---

-- for stationary shared mobility
CREATE TABLE shared_mobility_stations(
    id                  slug NOT NULL,
    origin              slug NOT NULL REFERENCES origins(id),
    name                TEXT NOT NULL,
    latitude            DOUBLE PRECISION NOT NULL,
    longitude           DOUBLE PRECISION NOT NULL,
    capacity            INTEGER,
    rental_uri_android  TEXT,
    rental_uri_ios      TEXT,
    rental_uri_web      TEXT,
    status              JSONB, -- realtime information about the stations status
    PRIMARY KEY(id, origin)
);

CREATE INDEX idx_shared_mobility_stations_name_trgm
    ON shared_mobility_stations
    USING GIN (name gin_trgm_ops);

CREATE TABLE shared_mobility_stations_original_ids(
    origin          slug NOT NULL REFERENCES origins(id),
    original_id     TEXT NOT NULL,
    id              slug NOT NULL,
    PRIMARY KEY(original_id, origin),
    FOREIGN KEY(id, origin) REFERENCES stops(id, origin)
);

CREATE INDEX ON shared_mobility_stations_original_ids(id, origin);

---/-------------------------\---
--|       ID GENERATION       |--
---\-------------------------/---

CREATE OR REPLACE FUNCTION set_shared_mobility_station_id()
RETURNS TRIGGER AS $$
DECLARE
    base_id TEXT;
    counter INTEGER := 1;
    final_id TEXT;
BEGIN
    IF NEW.id IS NULL THEN
        base_id := create_slug(NEW.name);
        final_id := base_id;

        WHILE EXISTS (SELECT 1 FROM shared_mobility_stations WHERE id = final_id) LOOP
            counter := counter + 1;
            final_id := concat(base_id, '-', counter);
        END LOOP;

        NEW.id := final_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER before_insert_generate_shared_mobility_station_id
    BEFORE INSERT ON shared_mobility_stations
    FOR EACH ROW
    EXECUTE FUNCTION set_shared_mobility_station_id();


---/-------------------------\---
--|        EXAMPLE DATA       |--
---\-------------------------/---

INSERT INTO origins(name, priority)
VALUES ('gbfs-donkey-kiel', 3);

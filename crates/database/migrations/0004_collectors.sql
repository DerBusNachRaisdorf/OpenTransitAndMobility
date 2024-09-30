
-- origins
INSERT INTO
    origins(id, name, priority)
VALUES
    ('migration', 'Migration', 1),
    ('delfi-germany', 'DELFI Germany', 2),
    ('gtfs-de', 'GTFS by Patrick Brosi', 3),
    ('connect-info-nah-sh', 'connect-info NAH.SH', 4),
    ('donkey-republic-kiel', 'Donkey Republic Kiel', 5),
    ('db-timetables', 'DB Timetables', 6);

-- collectors
INSERT INTO
    collectors(origin, kind, is_active, state)
VALUES
    -- db timetables
    (
        'db-timetables',
        'DB Timetables',
        true,
        '{"credentials": {"clientId": "INSERT_SECRET_HERE", "clientSecret": "INSERT_SECRET_HERE", "requests_per_minute": 60}, "stations": []}'
    ),
    -- gtfs delfi germany
    (
        'delfi-germany',
        'GTFS Schedule',
        false,
        '{"url": "https://de.data.public-transport.earth/gtfs-germany.zip"}'
    ),
    -- gtfs.de germany
    (
        'gtfs-de',
        'GTFS Schedule',
        false,
        '{"url": "https://download.gtfs.de/germany/free/latest.zip"}'
    ),
    (
        'gtfs-de',
        'GTFS Realtime',
        false,
        '{"url": "https://realtime.gtfs.de/realtime-free.pb", "updateInterval": {"secs": 60, "nanos": 0}}'
    ),
    -- gtfs connect-info nah.sh
    (
        'connect-info-nah-sh',
        'GTFS Schedule',
        true,
        '{"url": "https://www.connect-info.net/opendata/gtfs/nah.sh/rjqfrkqhgu"}'
    ),
    (
        'connect-info-nah-sh',
        'GTFS Realtime',
        true,
        '{"url": "https://gtfsr.vbn.de/DluWlw1jMRKr", "updateInterval": {"secs": 60, "nanos": 0}}'
    ),
    -- gbfs doneky kiel
    (
        'donkey-republic-kiel',
        'GBFS Stations',
        true,
        '{"url": "https://stables.donkey.bike/api/public/gbfs/2/donkey_kiel/en/station_information.json"}'
    ),
    (
        'donkey-republic-kiel',
        'GBFS Status',
        true,
        '{"url": "https://stables.donkey.bike/api/public/gbfs/2/donkey_kiel/en/station_status.json"}'
    );

INSERT INTO agencies(origin, name, website)
VALUES ('migration', 'erixx', 'https://erixx.de');

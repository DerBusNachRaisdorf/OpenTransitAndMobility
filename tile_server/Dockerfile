# +------------------------------+
# |            BUILD             |
# +------------------------------+

# Image for converting pbf to pmtiles
FROM ghcr.io/systemed/tilemaker:master AS build

COPY scripts/build-deps.sh build-deps.sh

# install build dependecies
RUN ./build-deps.sh

# Get OSM-file
ADD https://download.geofabrik.de/europe/germany/schleswig-holstein-latest.osm.pbf /data/
#ADD https://download.geofabrik.de/europe/germany-latest.osm.pbf /data/
# Get shapefile
ADD https://osmdata.openstreetmap.de/download/water-polygons-split-4326.zip .
RUN unzip -j water-polygons-split-4326.zip -d coastline/

# Copy configs
COPY config-openmaptiles.json config.json
COPY process-openmaptiles.lua process.lua

# Convert
RUN /usr/src/app/resources/docker-entrypoint.sh \
    /data/schleswig-holstein-latest.osm.pbf \
    #/data/germany-latest.osm.pbf \
    --config config.json \
    --process process.lua \
    --output /data/output.pmtiles

# +------------------------------+
# |            FINAL             |
# +------------------------------+

# Server for serving the vector tiles
FROM ghcr.io/maplibre/martin:latest

WORKDIR /data

# Copy build stage output
COPY --from=build /data/output.pmtiles output.pmtiles

# Copy font files
COPY fonts/ fonts/

ENTRYPOINT [ "martin", "--font", "fonts/klokantech", "--font", "fonts/metropolis", "/data/output.pmtiles" ]

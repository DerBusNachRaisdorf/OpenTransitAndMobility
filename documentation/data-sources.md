# Information Sources

A brief overview of what data, standards and publishers are available in the domain
of public transport and mobility. This list aims to compile public transport data
from all world, but is currently focused on Northern Germany, especially the
Holsteinische Schweiz (Bad Malente-Gremsmühlen, Eutin, Plön, Lütjenburg), Kiel and
Lübeck.

## Standards

Overview of relevant standards used for the exchange of public transport and mobility
data.

- [General Transit Feed Specification (GTFS)](https://gtfs.org/)
  - Standard for exchange of both static timetable information (GTFS Schedule) and
    real-time updates (GTFS Realtime)
  - **Subject**: Public Transport
  - **Data Format**: Multiple CSV files, usually in zip archive (GTFS Schedule) and
    Protobuf (GTFS Realtime)
- [General Bike Feed Specification (GBFS)](https://gbfs.org/)
  - Standard for the exchange of shared mobility information regarding station
    locations and status (occupation, availability, etc.)
  - **Data Format**: JSON
  - **Subject**: Shared Mobility
- [Transmodel](https://transmodel-cen.eu/)
  - **Subject**: Public Transport
  - [NeTEx](https://transmodel-cen.eu/index.php/netex/)
    - CEN Technical Standard for exchanging Public Transport schedules and related data
    - **Data Format**: XML
    - **Repository**: [GitHub](https://github.com/NeTEx-CEN/NeTEx)
  - [SIRI](https://transmodel-cen.eu/index.php/siri/)
    - Standard for the exchange of real-time information about public transportation
    - **Data Formats**: XML, (JSON)
- [HaCon Fahrplan-Auskunfts-System (HAFAS)](https://www.hacon.de/)
  - Used popular in Germany, used by various transport agencies and the Deutsche Bahn.

## Lists

- [awesome-transit](https://github.com/MobilityData/awesome-transit)

## Platforms

Overview of data platforms, where data regarding specific subjects is compiled and
shared.

- [Mobility Database](https://mobilitydatabase.org/)

### Europe

- [transport.rest](https://transport.rest/)
  - Compilation of different public transport related APIs, including a wrapper
    around the Deutsche Bahn HAFAS API.
  - **Scope**: Europe, Germany, Poland, Flixbus, ~~Schleswig-Holstein (Germany)~~,
    Berlin (Germany), Brandenburg (Germany), Nottingham City (England); Public Transport
  - **Agencies**: [Deutsche Bahn](https://www.deutschebahn.com/),
    [Verkehrsverbund Berlin-Brandenburg (VBB)](https://www.vbb.de/),
    [~~NAH.SH~~](https://www.nah.sh/), [Flixbus](https://www.flixbus.de/),
    [Nottingham City Transport](https://www.nctx.co.uk/)

### Germany

- [Mobilithek](https://mobilithek.info/)
  - Platform for the exchange of information by mobility agencies, infrastructure
    operators, traffic authorities
  - **Provider**: Bundesministerium für Digitales und Verkehr
  - **Scope**: Germany; Public Transport, Shared Mobility, Mobility
  - **Subjects**: Stops, Timetables, Real-time Information, Shared Mobility Stations,
    etc.
- [OpenData ÖPNV](https://www.opendata-oepnv.de/ht/de/willkommen)
  - **Provider**: Verkehrsverbund Rhein-Ruhr AöR
  - **Scope**: Germany; Public Transport
  - **Subjects**: Stops, Timetables, Real-time Information, etc.

### Northern Germany
- [Open-Data Schleswig-Holstein](https://opendata.schleswig-holstein.de/dataset)
  - Open data platform of the German state of Schleswig-Holstein.
  - **Provide**: Der Ministerpräsident des Landes Schleswig-Holstein
  - **Scope**: Schleswig-Holstein, Germany; various, including Public Transport,
    Shared Mobility, Infrastructure, etc.

## Publishers, Associations and Organizations

### Germany

- [Durchgängige ELektronische Fahrgastinformation (DELFI)](https://www.delfi.de/)

## Sources

- [Transit Land API](https://www.transit.land/documentation)

### Germany

- DELFI Datasets
  - Public Transport information for all of Germany
  - **Publisher**: [DELFI](https://www.delfi.de/)
  - **Scope**: Germany; Public Transport
  - **Standards**: GTFS, NeTEx, SIRI
  - **Timetables**:
    - GTFS Schedule
      - Contains about ~420.000 errors according to [mfdz.de](https://gtfs.mfdz.de/)
      - **Links**:
        - [Mobilithek, Deutschlandweite Sollfahrplandaten (GTFS)](https://mobilithek.info/offers/-2883874086141693018),
        - [Mobilithek, DELFI-Timetable GTFS](https://mobilithek.info/offers/552578819783815168),
        - [OpenData ÖPNV, Deutschlandweite Sollfahrplandaten (GTFS)](https://www.opendata-oepnv.de/ht/de/organisation/delfi/startseite?tx_vrrkit_view%5Baction%5D=details&tx_vrrkit_view%5Bcontroller%5D=View&tx_vrrkit_view%5Bdataset_name%5D=deutschlandweite-sollfahrplandaten-gtfs)
      - **Permanent Download Links**:
        - [public-transport.earth](https://de.data.public-transport.earth/gtfs-germany.zip)
        - Personal permanent download links can be created at the OpenData ÖPNV website.
          This requires creating an account.
    - NeTEx
      - **Links**
        - [Mobilithek, Deutschlandweite Sollfahrplandaten (NeTEX)](https://mobilithek.info/offers/-5402675120956083567)
        - [Mobilithek, DELFI-Timetable NeTEx](https://mobilithek.info/offers/552567007080263680)
        - [OpenData ÖPNV, Deutschlandweite Sollfahrplandaten (NeTEX)](https://www.opendata-oepnv.de/ht/de/organisation/delfi/startseite?tx_vrrkit_view%5Baction%5D=details&tx_vrrkit_view%5Bcontroller%5D=View&tx_vrrkit_view%5Bdataset_name%5D=deutschlandweite-sollfahrplandaten)
      - **Permanent Download Links**:
        - Personal permanent download links can be created at the OpenData ÖPNV website.
          This requires creating an account.
  - **Real-time Information**:
    - GTFS Realtime
      - Matches the GTFS Schedule feed
      - **Links**:
        - [Mobilithek, DELFI-Realtime GTFS-RT Trip Updates](https://mobilithek.info/offers/755009281410899968)
      - **Permanent Download Links**:
        - [vbn.de](https://gtfsr.vbn.de/ose3pqfh6x)
    - SIRI
      - Matches the NeTEx data set
      - **Links**:
        - [Mobilithek, DELFI-Realtime SIRI ET](https://mobilithek.info/offers/754669461266382848)
      - **Permanent Download Links**:
        - [delfi.de](https://extern.rcsued.delfi.de/20230621-mobilithek-siri-et/estimated-timetable.xml)
  - **Stops**:
    - **Links**:
      - [Mobilithek, Haltestellenverzeichnis](https://mobilithek.info/offers/-7074764409605819222)
      - [Mobilithek, Deutschlandweite Haltestellendaten](https://mobilithek.info/offers/-4580605166452085238)
      - [Mobilithek, Deutschlandweite Haltestellendaten (XML)](https://mobilithek.info/offers/631185048592105472)
      - [Mobilithek, Deutschlandweite Haltestellendaten (XML)](https://mobilithek.info/offers/-9101397721002771401)
      - [OpenData ÖPNV, Deutschlandweite Haltestellendaten](https://www.opendata-oepnv.de/ht/de/organisation/delfi/startseite?tx_vrrkit_view%5Baction%5D=details&tx_vrrkit_view%5Bcontroller%5D=View&tx_vrrkit_view%5Bdataset_name%5D=deutschlandweite-haltestellendaten)
      - [OpenData ÖPNV, Deutschlandweite Haltestellendaten (XML)](https://www.opendata-oepnv.de/ht/de/organisation/delfi/startseite?tx_vrrkit_view%5Baction%5D=details&tx_vrrkit_view%5Bcontroller%5D=View&tx_vrrkit_view%5Bdataset_name%5D=deutschlandweite-haltestellendaten-xml)
- [gtfs.de by Patrick Brosi](https://gtfs.de/)
  - Processed GTFS dataset based upon the DELFI NeTEx data set
  - Significant better data quality than the GTFS data set offered by DELFI, but
    lacks some information in the free version
  - **Scope**: Germany; Public Transport
  - **Timetables**:
    - **Standard**: GTFS Schedule
    - **Links**:
      - [Deutschland gesamt](https://gtfs.de/de/feeds/de_full/)
      - [Schienenfernverkehr Deutschland](https://gtfs.de/de/feeds/de_fv/)
      - [Öffentlicher Nahverkehr Deutschland](https://gtfs.de/de/feeds/de_nv/)
      - [Schienenregionalverkehr Deutschland](https://gtfs.de/de/feeds/de_rv/)
    - **Permanent Donwload Links**:
      - [Deutschland gesamt](https://download.gtfs.de/germany/free/latest.zip)
      - [Schienenfernvehrkehr Deutschland](https://download.gtfs.de/germany/fv_free/latest.zip)
      - [Öffentlicher Nahverkehr Deutschland](https://download.gtfs.de/germany/nv_free/latest.zip)
      - [Schienenregionalverkehr Deutschland](https://download.gtfs.de/germany/rv_free/latest.zip)
  - **Real-time Updates**:
    - Matches all of the GTFS Schedule feeds from [gtfs.de](https://gtfs.de/)
    - **Standard**: GTFS Realtime
    - **Links**:
      - [GTFS Realtime](https://gtfs.de/de/realtime/)
    - **Permanent Download Links**:
      - [GTFS Realtime](https://realtime.gtfs.de/realtime-free.pb)
- [DB Timetables API](https://developers.deutschebahn.com/db-api-marketplace/apis/product/timetables)
  - Proprietary API by the Deutsche Bahn
  - Provides schedule and real-time updates for all train stations operated by
    DB Station&Service
  - Very ugly API, one has to guess IDs, umlaute in station names are always problematic,
    sometimes stations are only found by eva numbers, sometimes only by ds100 numbers.
    Instead of linking to other stations by literal uris or ids, the route of trips
    is only provided as a string of station names, separated by "|" (pipe symbols).
  - **Format**: XML
- [DB StaDa - Station Data](https://developers.deutschebahn.com/db-api-marketplace/apis/product/stada)
  - Proprietary API by the Deutsche Bahn
  - Detailed information about (mostly) German railway stations.
  - **Format**: JSON
- [Other DB APIs](https://developers.deutschebahn.com/db-api-marketplace/apis/product)
  - Most are not really usable due to pricing or exclusive availability for railway
    transport companies or the like.

### Northern Germany

- [NAH.SH](https://nah.sh/) Connect
  - **Scope**: Schleswig-Holstein (Germany); Public Transport
  - **Timetables**:
    - **Standard**: GTFS Schedule
    - **Links**:
      - [Mobilithek, GTFS Datensatz Connect (Schleswig-Holstein)](https://mobilithek.info/offers/766317902476267520)
      - [OpenData ÖPNV, Fahrplandaten](https://www.opendata-oepnv.de/ht/de/datensaetze?tx_vrrkit_view%5Baction%5D=details&tx_vrrkit_view%5Bcontroller%5D=View&tx_vrrkit_view%5Bdataset_name%5D=fahrplandaten)
      - [Open-Data Schleswig-Holstein, Fahrplandaten](https://opendata.schleswig-holstein.de/dataset/fahrplandaten)
    - **Permanent Download Links**:
      - [connect-info.net](https://www.connect-info.net/opendata/gtfs/nah.sh/rjqfrkqhgu)
      - Personal permanent download links can be created at the OpenData ÖPNV website.
        This requires creating an account.
  - **Real-time Updates**:
    - Matches the above GTFS Schedule feed
    - **Standard**: GTFS Realtime
    - **Links**:
      - [Mobilithek, GTFS-RT Datensatz Connect (Schleswig-Holstein)](https://mobilithek.info/offers/766315425546817536)
    - **Permanent Download Links**
      - [vbn.de](https://gtfsr.vbn.de/DluWlw1jMRKr)
- [SprottenFlotte - KielRegion](https://www.kielregion.de/mobilitaetsregion/sprottenflotte/) ([Donkey Republic](https://www.donkey.bike/))
  - Information about bike station locations, availability, prices and occupancy
  - **Scope**: Kiel and surrounding (Schleswig-Holstein, Germany); Shared Mobility, Bikes
  - **Standard**: GBFS
  - **API Endpoint**: [https://stables.donkey.bike/api/public/gbfs/2/donkey_kiel/gbfs](https://stables.donkey.bike/api/public/gbfs/2/donkey_kiel/gbfs)
- [TIER escooter](https://www.tier.app/)
  - Information about e.g., current TIER escooter locations and state
  - **Scope**: Mainly Europe; escooter
  - **Official Docs**: https://api-documentation.tier-services.io/
  - **Better Docs**: https://github.com/ubahnverleih/WoBike/blob/master/Tier.md
  - **API-Key v1 & v2**: header `X-Api-Key: bpEUTJEBTf74oGRWxaIcW7aeZMzDDODe1yBoSxi2`
  - **API-Endpoints**:
    - All vehicles in Kiel: `https://platform.tier-services.io/v2/vehicle?zoneId=KIEL`

## Libraries and Tools
- [schmiddi-on-mobile/railway-backend](https://gitlab.com/schmiddi-on-mobile/railway-backend)

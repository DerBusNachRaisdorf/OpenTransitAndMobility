# Public Transport Server

Unified service providing information about public transport gathered from various sources.

## How to run

In order to start the server, two additional steps are required.

1. Populate an `.env` file similar to the `.env.example` file.
2. Insert your DB Timetables API secrets in `crates/database/migrations/0004_collectors.sql`.
The client ID and client secret can be obtained at the [DB API Marketplace](https://developers.deutschebahn.com/db-api-marketplace/apis/product/timetables).

## Documentation

- [Related Work](documentation/related-work.md)
- [(incomplete) List of Relevant Data Sources and Standards](documentation/data-sources.md)

## UI Inspiration
- [Search Bar](https://codepen.io/mey_mnry/pen/QWqPvox)

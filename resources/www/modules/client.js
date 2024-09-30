import { SafariCompatibleDate } from "./utililty.js";

export class Agency {
  #json;

  constructor(json) {
    this.#json = json;
  }

  get name() {
    return this.#json.name;
  }

  get website() {
    return this.#json.website;
  }

  get phoneNumber() {
    return this.#json.phoneNumber;
  }

  get email() {
    return this.#json.email;
  }

  get fareUrl() {
    return this.#json.fareUrl;
  }
}

export class StopTime {
  #json;

  constructor(json) {
    this.#json = json;
  }

  get stopSequence() {
    return this.#json.stopSequence;
  }

  get stopName() {
    return this.#json.stopName;
  }

  get arrivalTime() {
    return this.#json?.arrivalTime
      ? new SafariCompatibleDate(this.#json.arrivalTime)
      : null;
  }

  get departureTime() {
    return this.#json?.departureTime
      ? new SafariCompatibleDate(this.#json.departureTime)
      : null;
  }

  get stopHeadsign() {
    return this.#json.stopHeadsign;
  }

  get interestFlag() {
    return this.#json.interestFlag;
  }

  get location() {
    return this.#json.location;
  }
}

export class Trip {
  #json;

  constructor(json) {
    this.#json = json;
  }

  get tripId() {
    return this.#json.tripId;
  }

  get headsign() {
    return this.#json.headsign;
  }

  get shortName() {
    return this.#json.shortName;
  }

  get stops() {
    return this.#json.stops.map((stopTimeJson) => new StopTime(stopTimeJson));
  }

  get stopOfInterest() {
    return new StopTime(this.#json.stopOfInterest);
  }

  get line() {
    if (this.#json.line) {
      return new Line(this.#json.line);
    }
    return null;
  }

  get agency() {
    if (this.#json.agency) {
      return new Agency(this.#json.agency);
    }
    return null;
  }
}

export class Line {
  #json;

  constructor(json) {
    this.#json = json;
  }

  get name() {
    return this.#json.name;
  }

  get kind() {
    return this.#json.kind;
  }
}

export class Stop {
  #json;

  constructor(json) {
    this.#json = json;
  }

  get name() {
    return this.#json.name;
  }

  get description() {
    return this.#json.description;
  }

  get longitude() {
    return this.#json.location?.longitude;
  }

  get latitude() {
    return this.#json.location?.latitude;
  }

  get distanceKm() {
    return this.#json.distanceKm;
  }

  get platformCode() {
    return this.#json.platformCode ?? null;
  }

  async getNearby() {
    let links = this.#json.links;
    for (let link of links) {
      if (link.rel == "nearby") {
        return fetch(link.href)
          .then((response) => response.json())
          .then((json) => json.data.map((stop) => new Stop(stop)));
      }
    }
    return null;
  }

  async getLines() {
    let links = this.#json.links;
    for (let link of links) {
      if (link.rel == "lines") {
        return fetch(link.href)
          .then((response) => response.json())
          .then((json) => json.data.map((line) => new Line(line)));
      }
    }
    return null;
  }

  async getTrips() {
    // TODO: take dateTime range
    let links = this.#json.links;
    for (let link of links) {
      if (link.rel == "trips") {
        return fetch(link.href)
          .then((response) => response.json())
          .then((json) => json.data.map((trip) => new Trip(trip)));
      }
    }
    return null;
  }
}

export class Nearby {
  #json;

  constructor(json) {
    this.#json = json;
  }

  get sharedMobilityStations() {
    return this.#json.sharedMobilityStations;
  }

  get stops() {
    return this.#json.stops.map((stop) => new Stop(stop));
  }

  get lines() {
    return this.#json.lines.map((line) => new Line(line));
  }

  get trips() {
    return this.#json.trips.map((trip) => new Trip(trip));
  }

  /**
   * @returns {EventSource}
   */
  realtime() {
    let links = this.#json.links;
    for (let link of links) {
      if (link.rel == "realtime") {
        return new EventSource(link.href);
      }
    }
    return null;
  }
}

function formatDateToString(date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0"); // Months are zero-based
  const day = String(date.getDate()).padStart(2, "0");
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  const seconds = String(date.getSeconds()).padStart(2, "0");

  return `${year}-${month}-${day}T${hours}:${minutes}:${seconds}`;
}

export class Client {
  async fetchStop(id) {
    return fetch(`/api/v1/stops/${id}`)
      .then((response) => response.json())
      .then((json) => new Stop(json));
  }

  async fetchStops() {
    return fetch(`/api/v1/stops`)
      .then((response) => response.json())
      .then((json) => json.data.map((stop) => new Stop(stop)));
  }

  async fetchStopsNearby(latitude, longitude, radius) {
    return fetch(
      `/api/v1/stops/nearby?latitude=${latitude}&longitude=${longitude}&radius=${radius}`,
    )
      .then((response) => response.json())
      .then((json) => json.data.map((stop) => new Stop(stop)));
  }

  async fetchNearby(latitude, longitude, radius, start = null) {
    start = start ?? new Date();
    return fetch(
      `/api/v1/nearby?latitude=${latitude}&longitude=${longitude}&radius=${radius}&start=${formatDateToString(start)}`,
    )
      .then((response) => response.json())
      .then((json) => new Nearby(json));
  }

  async searchStop(pattern) {
    return fetch(`/api/v1/stops/search/${pattern.toLocaleLowerCase()}`)
      .then((response) => response.json())
      .then((json) =>
        json.data
          .filter((stop) => stop.latitude && stop.longitude)
          .filter((v, i, a) => a.findIndex((x) => x.name === v.name) === i),
      );
  }
}

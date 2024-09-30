import { Client } from "./modules/client.js";
import { kilometersToReadableText, timeString } from "./modules/utililty.js";
import { SearchBar, SearchBarItem } from "./modules/ui/searchbar.js";

const RADIUS_KM = 0.3;

const URL_PARAMS = new URLSearchParams(window.location.search);
const DEBUG = URL_PARAMS.get("debug");

var realtimeSource = null;

let client = new Client();

var map = new maplibregl.Map({
  container: "map",
  style: "./mapstyle-light.json", // stylesheet location
  center: [10.551556, 54.167111], // starting position [lng, lat],
  //maxBounds: [7.465, 53.35, 11.8, 55.145], // schleswig holstein
  //maxBounds: [5.98865807458, 47.3024876979, 15.0169958839, 54.983104153], // germany
  zoom: 8, // starting zoom
  maxZoom: 20,
  minZoom: 1, //  8,
  attributionControl: false,
});

map.on("load", async () => {
  const image = await map.loadImage("/images/stop-marker.png");

  map.addImage("custom-marker", image.data);

  const urlParams = new URLSearchParams(window.location.search);
  const showAll = urlParams.get("showall");
  let stops = showAll ? await client.fetchStops() : [];
  let features = stops.map((stop) => {
    return {
      type: "Feature",
      geometry: {
        type: "Point",
        coordinates: [stop.longitude, stop.latitude],
      },
      properties: {
        name: stop.name + " " + (stop.platformCode ?? ""),
        distance: stop.distance,
      },
    };
  });

  // stop source
  map.addSource("stops", {
    type: "geojson",
    data: {
      type: "FeatureCollection",
      features: features,
    },
  });

  // stop layer
  map.addLayer({
    id: "stops",
    type: "symbol",
    source: "stops",
    layout: {
      "icon-image": "custom-marker",
      "text-field": ["get", "name"],
      "text-font": ["Metropolis Regular"],
      "text-offset": [0, 1.25],
      "text-anchor": "top",
    },
  });

  // Add the circle as a GeoJSON source
  map.addSource("location-radius", {
    type: "geojson",
    data: {
      type: "FeatureCollection",
      features: [],
    },
  });

  // circle fill layer
  map.addLayer({
    id: "location-radius",
    type: "fill",
    source: "location-radius",
    paint: {
      "fill-color": "#AA2255",
      "fill-opacity": 0.1,
    },
  });

  // circle outline layer
  map.addLayer({
    id: "location-radius-outline",
    type: "line",
    source: "location-radius",
    paint: {
      "line-color": "#AA2255",
      "line-opacity": 0.3,
      "line-width": 3,
    },
  });

  // trip route source
  map.addSource("trip-route", {
    type: "geojson",
    data: {
      type: "FeatureCollection",
      features: [],
    },
  });

  // trip route layer
  map.addLayer({
    id: "trip-route",
    type: "line",
    source: "trip-route",
    layout: {
      "line-join": "round",
      "line-cap": "round",
    },
    paint: {
      "line-color": "#AA225566",
      "line-width": 5,
    },
  });

  // trip route stop source
  map.addSource("trip-route-stops", {
    type: "geojson",
    data: {
      type: "FeatureCollection",
      features: [],
    },
  });

  // trip route stop layer
  map.addLayer({
    id: "trip-route-stops",
    type: "symbol",
    source: "trip-route-stops",
    layout: {
      "icon-image": "custom-marker",
      "text-field": ["get", "displayText"],
      "text-font": ["Metropolis Regular"],
      "text-offset": [0, 1.25],
      "text-anchor": "top",
    },
  });

  if (navigator.geolocation) {
    navigator.geolocation.getCurrentPosition((position) => {
      if (!alreadyNaviagtedToLocation) {
        navigateToLocation(position.coords.latitude, position.coords.longitude);
      }
    });
  }
});

await map.once("load");
var currentLocation = null;

// datetime
var dateTimeInput = document.getElementById("datetime-input");

function setDateTimeValue(value) {
  let date = new Date(value);
  date.setMinutes(date.getMinutes() - date.getTimezoneOffset());
  dateTimeInput.value = date.toISOString().slice(0, 16);
}

setDateTimeValue(new Date());

dateTimeInput.addEventListener("input", () => {
  if (currentLocation != null) {
    navigateToLocation(
      currentLocation.latitude,
      currentLocation.longitude,
      new Date(dateTimeInput.value),
    );
  }
});

// sidebar
var sidebarPadding = 0;
const leftSidebarButton = document.getElementById("sidebar-toggle-button");
const leftSidebar = document.getElementById("left-sidebar");
leftSidebarButton.addEventListener("click", () => toggleSidebar());

function toggleSidebar(ease = true) {
  const classes = leftSidebar.className.split(" ");
  const collapsed = classes.indexOf("collapsed") !== -1;

  if (collapsed) {
    showSidebar(ease);
  } else {
    hideSidebar(ease);
  }
}

function showSidebar(ease = true) {
  const classes = leftSidebar.className.split(" ");
  const collapsed = classes.indexOf("collapsed") !== -1;
  if (collapsed) {
    classes.splice(classes.indexOf("collapsed"), 1);
    sidebarPadding = leftSidebar.getBoundingClientRect().width;
    if (ease) {
      map.easeTo({
        padding: { left: sidebarPadding },
        duration: 1000, // In ms, CSS transition duration property for the sidebar matches this value
      });
    }
    // Update the class list on the element
    leftSidebar.className = classes.join(" ");
  }
}

function hideSidebar(ease = true) {
  const classes = leftSidebar.className.split(" ");
  const collapsed = classes.indexOf("collapsed") !== -1;
  if (!collapsed) {
    sidebarPadding = 0;
    classes.push("collapsed");
    if (ease) {
      map.easeTo({
        padding: { left: sidebarPadding },
        duration: 1000,
      });
    }
    // Update the class list on the element
    leftSidebar.className = classes.join(" ");
  }
}

var currentMarker = null;
map.on("click", async (e) => {
  const { lngLat } = e;
  const { lng, lat } = lngLat;

  await navigateToLocation(lat, lng);
});

function collapsibleOnClick(elem) {
  for (let c of document.querySelectorAll(".collapsible")) {
    if (c != elem) {
      c.classList.remove("active");
      let content = c.nextElementSibling;
      content.style.maxHeight = null;
    }
  }

  // with animation:
  let expanded = false;
  elem.classList.toggle("active");
  var content = elem.nextElementSibling;
  if (content.style.maxHeight) {
    content.style.maxHeight = null;
  } else {
    collapsibleShowContent(content);
    expanded = true;
  }
  return expanded;
}

function collapsibleShowContent(content, doExpand = true) {
  if (!doExpand && !content.style.maxHeight) {
    return;
  }
  content.style.maxHeight = content.scrollHeight + "px";
}

function removeMarker() {
  if (currentMarker) {
    currentMarker.remove();
  }
}

function setMarker(lng, lat) {
  removeMarker();
  currentMarker = new maplibregl.Marker({
    color: "#AA2255",
  })
    .setLngLat([lng, lat])
    .addTo(map);
}

const loader = document.getElementById("loader");
const loaderContainer = document.getElementById("loader-container");
const content = document.getElementById("left-sidebar-content");

function showLoader() {
  loaderContainer.style = undefined;
  content.style.display = "none";
}

function hideLoader() {
  loaderContainer.style.display = "none";
  content.style.display = "block";
  content.scrollTop = 0;
}

function zoomToCurrentLocation() {
  const ZOOM = 14;
  showSidebar(false);
  map.easeTo({
    center: [currentLocation.longitude, currentLocation.latitude],
    zoom: ZOOM,
    bearing: 0,
    duration: 1000,
    essential: true,
    padding: { left: sidebarPadding },
  });
}

// used to determine wether to navigate to device location or not.
var alreadyNaviagtedToLocation = false;

async function navigateToLocation(lat, lng, datetime = null) {
  datetime = datetime ?? new Date();
  setDateTimeValue(datetime);

  alreadyNaviagtedToLocation = true;
  currentLocation = { latitude: lat, longitude: lng };
  leftSidebar.classList.remove("hidden");

  hideTripRoute();
  showLoader();

  // zoom to location
  zoomToCurrentLocation();

  setMarker(lng, lat);

  // show radius
  showRadius(lng, lat, RADIUS_KM);

  // fetch nearby stops
  realtimeSource?.close();
  let nearby = await client.fetchNearby(lat, lng, RADIUS_KM, datetime);

  displayNearby(nearby);

  realtimeSource = nearby.realtime();
  realtimeSource?.addEventListener("message", function (event) {
    let realtimeData = JSON.parse(event.data);
    for (const update of realtimeData.tripUpdates) {
      updateTripUIElement(update);
    }
  });
}

async function displayNearby(nearby) {
  hideLoader();

  // display stops on map
  map.getSource("stops").setData({
    type: "FeatureCollection",
    features: nearby.stops.map((stop) => {
      return {
        type: "Feature",
        geometry: {
          type: "Point",
          coordinates: [stop.longitude, stop.latitude],
        },
        properties: {
          name: stop.name + " " + (stop.platformCode ?? ""),
          distance: stop.distance,
        },
      };
    }),
  });

  // show stops at sidebar
  let sidebarContent = document.getElementById("left-sidebar-content");
  sidebarContent.innerHTML = "";

  let descriptionContainer = document.createElement("div");
  descriptionContainer.className = "description-container";
  descriptionContainer.innerHTML = nearby.stops
    .filter((stop) => stop.description)
    .map((stop) => stop.description)
    .join("</br>\n");
  sidebarContent.appendChild(descriptionContainer);

  let lineContainer = document.createElement("div");
  lineContainer.className = "line-container";
  let lineNames = nearby.lines.slice(0, 10).map((line) => line.name);
  if (lineNames.length != nearby.lines.length) {
    lineNames.push("...");
  }
  for (let line of lineNames) {
    let lineElement = document.createElement("b");
    lineElement.className = "line-inactive";
    lineElement.innerText = line;
    lineContainer.appendChild(lineElement);
  }
  sidebarContent.appendChild(lineContainer);

  for (let station of nearby.sharedMobilityStations) {
    let header = document.createElement("button");
    header.type = "button";
    header.className = "collapsible";
    header.style = "background-image: url('images/bike.svg');";

    // card
    let cardElement = document.createElement("div");
    cardElement.className = "card";
    header.appendChild(cardElement);

    // title
    let titleElement = document.createElement("h4");
    titleElement.innerHTML = station.name;
    cardElement.appendChild(titleElement);

    // content
    let contentElement = document.createElement("div");
    contentElement.className = "collapsible-content";

    // Capacity
    let stopElement = document.createElement("div"); // button
    stopElement.className = "stop-time";

    let stopNameElement = document.createElement("a");
    stopNameElement.className = "stop-time-name";
    stopNameElement.innerHTML = `<b>Kapazität</b>`;
    stopElement.appendChild(stopNameElement);

    let stopArrivalElement = document.createElement("a");
    stopArrivalElement.className = "stop-time-arrival";
    stopArrivalElement.innerHTML = String(station.capacity);
    stopArrivalElement.style.paddingRight = "20px";
    stopElement.appendChild(stopArrivalElement);

    // Capacity
    let bikesAvailable = document.createElement("div"); // button
    bikesAvailable.className = "stop-time";

    let bikesAvailableName = document.createElement("a");
    bikesAvailableName.className = "stop-time-name";
    bikesAvailableName.innerHTML = `<b>Verfügbar</b>`;
    bikesAvailable.appendChild(bikesAvailableName);

    let bikesAvailableContent = document.createElement("a");
    bikesAvailableContent.className = "stop-time-arrival";
    bikesAvailableContent.innerHTML = String(station.status.numBikesAvailable);
    bikesAvailableContent.style.paddingRight = "20px";
    bikesAvailable.appendChild(bikesAvailableContent);

    contentElement.appendChild(stopElement);
    contentElement.appendChild(bikesAvailable);

    sidebarContent.appendChild(header);
    sidebarContent.appendChild(contentElement);

    header.addEventListener("click", () => {
      collapsibleOnClick(header);
    });
  }

  for (let trip of nearby.trips) {
    // stop ui element
    let headerElement = document.createElement("button");
    headerElement.type = "button";
    headerElement.className = "collapsible";
    if (trip.line.kind == "rail" && trip.line.name == "Intercity-Express") {
      headerElement.style = "background-image: url('images/ice.svg');";
    } else if (trip.line.kind == "rail") {
      headerElement.style = "background-image: url('images/train.svg');";
    } else if (trip.line.kind == "bus") {
      headerElement.style = "background-image: url('images/bus.svg');";
    }
    displayTrip(headerElement, trip);
    let contentElement = document.createElement("div");
    contentElement.className = "collapsible-content";
    for (let stop of trip.stops.filter((stop) => stop.stopName)) {
      let stopElement = document.createElement("button");
      stopElement.className = "stop-time";
      stopElement.addEventListener("click", () => {
        navigateToLocation(
          stop.location.latitude,
          stop.location.longitude,
          stop.arrivalTime,
        );
      });

      let stopNameElement = document.createElement("a");
      stopNameElement.className = "stop-time-name";
      stopNameElement.innerHTML = stop.interestFlag
        ? `<b>${stop.stopName}</b>`
        : stop.stopName;
      stopElement.appendChild(stopNameElement);

      let stopArrivalElement = document.createElement("a");
      stopArrivalElement.className = "stop-time-arrival";
      stopArrivalElement.innerHTML = stop.arrivalTime
        ? timeString(stop.arrivalTime)
        : "";
      stopElement.appendChild(stopArrivalElement);

      contentElement.appendChild(stopElement);
    }
    sidebarContent.appendChild(headerElement);
    sidebarContent.appendChild(contentElement);

    // stop ui element on click
    headerElement.addEventListener("click", () => {
      if (collapsibleOnClick(headerElement)) {
        displayTripRoute(trip);
        headerElement.scrollIntoView({ behavior: "smooth", block: "start" });
      } else {
        hideTripRoute();
        zoomToCurrentLocation();
      }
    });
  }

  if (nearby.trips.length == 0) {
    let info = document.createElement("div");
    info.innerText = "Hier fährt gerade nix.";
    info.style.textAlign = "center";
    info.style.justifySelf = "center";
    info.style.flexGrow = 1;
    info.style.width = "100%";
    info.style.fontSize = "16px";
    sidebarContent.appendChild(info);
  }
}

var tripUIElements = new Map();

const yyymmddFromDate = (date) =>
  `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;

const tripUIElementIDFromTrip = (trip) =>
  `${trip.tripId}-${yyymmddFromDate(trip.stops[0].departureTime ?? new Date())}`;

const tripUIElementID = (id, tripStartDate) => `${id}-${tripStartDate}`;

/**
 * displays a trip
 */
function displayTrip(parent, trip) {
  let lineName = trip.line?.name;
  let title = lineName
    ? `<b class="line">${lineName}</b> ${trip.headsign}`
    : trip.headsign;

  // card
  let cardElement = document.createElement("div");
  cardElement.className = "card";
  parent.appendChild(cardElement);

  // title
  let titleElement = document.createElement("h4");
  titleElement.innerHTML = title;
  cardElement.appendChild(titleElement);

  // DEBUG: id
  if (DEBUG) {
    let debugId = document.createElement("div");
    debugId.className = "debug-id";
    debugId.innerHTML = trip.tripId;
    cardElement.appendChild(debugId);
  }

  // info table
  let body = document.createElement("div");
  body.className = "trip-table";
  cardElement.appendChild(body);
  let table = document.createElement("table");
  body.appendChild(table);
  let tbody = document.createElement("tbody");
  table.appendChild(tbody);

  const createRow = (name, html) => {
    let row = document.createElement("tr");
    let nameElement = document.createElement("td");
    nameElement.style.paddingRight = "20px";
    nameElement.innerHTML = `<b>${name}</b>`;
    row.appendChild(nameElement);
    let contentElement = document.createElement("td");
    contentElement.innerHTML = html;
    row.appendChild(contentElement);
    let contentCorrectionElement = document.createElement("b");
    contentCorrectionElement.style.paddingLeft = "10px";
    contentElement.appendChild(contentCorrectionElement);
    tbody.appendChild(row);
    return {
      row: row,
      planned: contentElement,
      realtime: contentCorrectionElement,
    };
  };

  // at stop
  let atStop = createRow("Haltestelle", trip.stopOfInterest.stopName);
  if (!trip.stopOfInterest?.stopName) {
    atStop.row.style.display = "none";
  }

  // arrival
  let arrivalTime = createRow(
    "Ankunft",
    trip.stopOfInterest?.arrivalTime
      ? timeString(trip.stopOfInterest.arrivalTime)
      : "",
  );
  if (!trip.stopOfInterest?.arrivalTime) {
    arrivalTime.row.style.display = "none";
  }

  // departure
  let departureTime = createRow(
    "Abfahrt",
    trip.stopOfInterest?.departureTime
      ? timeString(trip.stopOfInterest.departureTime)
      : "",
  );
  if (!trip.stopOfInterest?.departureTime) {
    departureTime.row.style.display = "none";
  }

  tripUIElements.set(tripUIElementIDFromTrip(trip), {
    trip: trip,
    title: titleElement,
    atStop: atStop,
    arrivalTime: arrivalTime,
    departureTime: departureTime,
  });
}

function updateTripUIElement(tripUpdate) {
  let id = tripUIElementID(tripUpdate.id.tripId, tripUpdate.id.tripStartDate);
  if (!tripUIElements.has(id)) {
    return;
  }

  let uiElement = tripUIElements.get(id);

  // stop of interest
  let stopOfInterestUpdate = tripUpdate.stops.find((stopTimeUpdate) => {
    return (
      stopTimeUpdate.scheduledStopSequence ==
      uiElement.trip.stopOfInterest.stopSequence
    );
  });
  if (stopOfInterestUpdate) {
    uiElement.arrivalTime.realtime.innerText = stopOfInterestUpdate?.arrivalTime
      ? timeString(new Date(stopOfInterestUpdate.arrivalTime))
      : "";
    uiElement.departureTime.realtime.innerText =
      stopOfInterestUpdate?.departureTime
        ? timeString(new Date(stopOfInterestUpdate.departureTime))
        : "";
  }
}

function displayTripRoute(trip) {
  let stopsWithLocation = trip.stops.filter((stop) => stop.location);
  let geojson = {
    type: "FeatureCollection",
    features: [
      {
        type: "Feature",
        geometry: {
          type: "LineString",
          properties: {},
          coordinates: stopsWithLocation.map((stop) => [
            stop.location.longitude,
            stop.location.latitude,
          ]),
        },
      },
    ],
  };

  // display route
  map.getSource("trip-route").setData(geojson);

  // display route stops
  map.getSource("trip-route-stops").setData({
    type: "FeatureCollection",
    features: stopsWithLocation.map((stop) => {
      let displayText =
        (stop.stopName ? stop.stopName + " " : "") +
        (stop.arrivalTime ? `(${timeString(stop.arrivalTime)})` : "");

      return {
        type: "Feature",
        geometry: {
          type: "Point",
          coordinates: [stop.location.longitude, stop.location.latitude],
        },
        properties: { displayText: displayText },
      };
    }),
  });

  // fit route
  let coordinates = geojson.features[0].geometry.coordinates;
  const bounds = coordinates.reduce(
    (bounds, coord) => {
      return bounds.extend(coord);
    },
    new maplibregl.LngLatBounds(coordinates[0], coordinates[0]),
  );

  map.fitBounds(bounds, {
    padding: 120,
  });
}

function hideTripRoute() {
  map.getSource("trip-route").setData({
    type: "FeatureCollection",
    features: [],
  });

  map.getSource("trip-route-stops").setData({
    type: "FeatureCollection",
    features: [],
  });
}

function hideEverything() {
  displayStops([]);
  hideRadius();
  removeMarker();
}

function showRadius(lat, long, radius) {
  // Generate a polygon using turf.circle.
  // See https://turfjs.org/docs/api/circle
  let radiusCenter = [lat, long];
  const options = {
    steps: 64,
    units: "kilometers",
  };
  const circle = turf.circle(radiusCenter, radius, options);
  map.getSource("location-radius").setData(circle);
}

function hideRadius() {
  map.getSource("location-radius").setData({
    type: "FeatureCollection",
    features: [],
  });
}

// search bar

var searchBar = new SearchBar(document.querySelector(".search-container"));
searchBar.placeholder = "Haltestelle suchen...";

searchBar.onItemSelected(async (_name, stop) => {
  await navigateToLocation(stop.latitude, stop.longitude);
});

searchBar.itemProvider(async (searchPattern) => {
  let stops =
    searchPattern.length != 0 ? await client.searchStop(searchPattern) : [];
  return stops.map((stop) => new SearchBarItem(stop.name, stop));
});

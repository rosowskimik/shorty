# Client

The UI component of the entire application.
This is a SSR application written in Rust using [Leptos framework](https://leptos.dev) with [Axum](https://github.com/tokio-rs/axum) integration.

## App startup

The app goes through three main steps when started.

1. The app parses basic configuration options and exists with an error if any of the required ones are missing.
Args can be provided either as command flags or (more safely) as environment variables.
If an option is passed as both, the flag value takes priority.

All cli options, their environment counterparts and default values can be checked by passing
`--help` to the client binary.

2. After parsing the configuration, the client will attempt to establish a connection with gRPC server,
which handles the shortening logic. The connection will be attempted up to 5 times after which,
if connection isn't established, the client will exit early with an error.

3. With connection established, the app will start up the Axum server responsible for
serving the frontend WASM application and the public facing API.

The frontend server is configured to use gzip compression to minimize served bundle size.

# API

Besides sevring static files (HTML, CSS, WASM), the frontend server exposes specialized routes
for handling the application logic

* `GET /s/<slug>` - the shortened URL route. When a matching `slug` is found, the server will respond with
`303 See Other` response, redirecting the client to the original URL. If no match is found, responds with generic `404 Not Found`.

* `GET, POST /api/*` - routes generated automatically by Leptos to handle its [server functions](https://book.leptos.dev/server/25_server_functions.html).

If none of those routes match, the server will check if the request corresponds to a static file
and serve it if match is found. Otherwise it responds with generic `404 Not Found`.

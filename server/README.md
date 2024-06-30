# Server

The backend component of the entire application
This is an async Rust server application using [Tokio](https://tokio.rs/) and [Tonic](https://github.com/hyperium/tonic) to implement a [gRPC](https://grpc.io/) URL shortener service.

## App startup

The app goes through three main steps when started.

1. The app parses basic configuration options and exists with an error if any of the required ones are missing.
Args can be provided either as command flags or (more safely) as environment variables.
If an option is passed as both, the flag value takes priority.

All cli options, their environment counterparts and default values can be checked by passing
`--help` to the server binary.

2. After parsing the configuration, the server will attempt to establish a connection with a database
used to store the URL mappings. If that fails, the server will exit early with an error.

3. With connection established, the app will start up the gRPC server responsible for
handling URL shortening requests.

## Shortener

### URL -> slug

When the server receives an URL shortening request, it goes through these steps:

1. The data contained within request body is checked first, to make sure that it is a valid URL.
If the validation fails, the server responds with an error

2. Next, a random, ASCII alphanumeric (a-z, A-Z and 0-9) string of preconfigured length is generated
to be used as the short slug.

3. After that, the server attempts to save the new mapping to the database

4. If that succeeds, the server sends back the response containing the newly generated slug.

### Slug -> URL

When the server receives a URL mapping fetch request, it goes through these steps:

1. The server checks if the requested URL mapping exists in the database,
using the slug provided in request body as a loookup key.

2. Depending on the lookup result, the server either responds with an OK message containing the original URL,
or not found error.

### Short slug generation

The slugs are generated using [xoshiro256++](https://prng.di.unimi.it/xoshiro256plusplus.c)
pseudo random number generator over uniformly distributed ASCII letters and numbers.

Each application thread gets it's own instance by storing it in thread local storage.
Each generator is lazily seeded on first usage with current system time in form of UNIX timestamp
(number of seconds since 1970-01-01 00:00:00 UTC)

## Database

[Redis](https://redis.io/), or any Redis compatible (like [keydb](https://docs.keydb.dev/)) database can be used.
The server will use this database to store the generated slug -> original URL mappings.

### Master - replicas setup

By default, if only one database connection is configured,
that database will be used for all requests by the server.
However, the server can also be configured to support master - replicas clusters.
If a replica database connection is configured,
all data modifying requests (writes) will be directed to the master database,
while immutable requests (reads) will sent to the replicas by the server.
This could possibly improve throughput, especially when reading data back,
since all read requests can be load balanced across many replicas.

This also doubles as an additional failover strategy:

* when a read request to a replica fails, the server will attempt to read directly from master
* in case of master failure, the replicas can still handle all read requests for existing data

> [!NOTE]
> The master - replicas cluster setup if the default when using provided `kubernetes` configuration

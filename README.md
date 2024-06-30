# Leptos + gRPC Url Shortener

Simple url shortener app written in Rust (Leptos app with hydration) + gRPC backend.
Used for Kubernetes course.

# Structure
The app is split into 3 main components:
* `client` frontend - end user UI written in Rust using Leptos with hydration for reactivity.
Main access point for rest of the app.
* `server` backend - a gRPC server written in Rust + tonic. Handles main app logic (creating shortened URLs and mapping shortened ones to original).
* database - Redis API compatible database which stores URL mappings

The gRPC `.proto` files are available in [proto directory](./proto).

(more details about specific elements is contained within their respective directories)

# Building

## Dependencies
To build both `client` and `server`, you will need:
- Rust nightly
- Protocol buffers compiler

additionally, to build the `client` app:
- [`cargo-leptos`](https://github.com/leptos-rs/leptos)

## Client

Clone the repo
```sh
git clone https://github.com/rosowskimik/shorty
cd shorty/client
```

Build the app
```sh
cargo leptos build --release
```

The resulting binary will be accessible at `release/client`,
while all the frontend elements (HTML, CSS, etc.) in `site` under your cargo target directory.

## Server

Clone the repo
```sh
git clone https://github.com/rosowskimik/shorty
cd shorty/server
```

Build the app
```sh
cargo build --release
```

Final binary will be accessible at `release/server` under your cargo target directory.

## Container images
Instead of building the app directly, you could also build them as container images.
Tagged releases for both [`client`](https://github.com/rosowskimik/shorty/pkgs/container/shorty-client) and
[`server`](https://github.com/rosowskimik/shorty/pkgs/container/shorty-server) are also published for use.

# Kubernetes
While it's possible to run the app standalone, it was mainly designed to run in a Kubernetes cluster.
All the files necessary for deploying the app are available in `.kuber` directory.

All settings relevant to app configuration are set using ConfigMaps (besides `server-token`, which is stored as a Secret)

> [!IMPORTANT]
> When using this app in your own deployment,
> make sure to change the `server-token` to your own in `server-secret.yaml` file.

The database is configured as master - replicas cluster, where all write requests from `server` are directed to the master, while reads are spread across all the replicas. Master also has exclusive access to PersistentVolume to keep data between master reboots / updates.

`Client`, `server` and database replicas are also setup to scale automatically under load using HorizontalPodAutoscaler.

> [!NOTE]
> When running in `minikube`, make sure the `metrics-server` addon is enabled:
> ```sh
> minikube addons enable metrics-server
> ```

`Client` app is the only service exposed to outside the cluster using LoadBalancer.
The app is configured to be accessible under `http://shor.tt:8080`.

> [!NOTE]
> If you're running the cluster locally using `minikube`, you will have to run `minikube tunnel`
> in order to make the `client` service available.
> You can also use [`setup_hosts.sh` script](./scripts/setup_hosts.sh) from this repo,
> which will modify your `/etc/hosts` file to make the app accessible using domain name, instead of ip address.
> ```sh
> ./scripts/setup_hosts.sh setup
> ```
> After you're done, you can run
> ```sh
> ./scripts/setup_hosts.sh clean
> ```
> to undo the changes done to your `/etc/hosts` file


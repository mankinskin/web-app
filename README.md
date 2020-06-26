# Budget App
### A social network for managing projects

## Development

This project is built using Rust and WebAssembly.

Backend:
- [Rocket](https://rocket.rs/) [server](server/)
- [rql](https://github.com/kaikalii/rql) [database](database/)

Frontend:
- [Seed](https://seed-rs.org/) Web Client Framework

### Building the client and running the server
```
  cd server/
  make run
```

`make run` will build the client to [wasm32-unknown-unknown](https://rustwasm.github.io/book/),
build the server and run it. The Makefile calls [cargo-make](https://github.com/sagiegurari/cargo-make)
internally, to perform this.

### Client/Server API
The server and client both import the [server/api](server/api) crate, which uses
[procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html)
to generate code for an RPC interface between clients and servers (the server
address is still fixed to localhost:8000 currently). This interface allows to
easily define functions on the server that can be invoked from the client. Using
this, database queries, authentication and all other client/server communication
is realized.

### Server

The server accepts requests by clients and accesses the database tables
through the DatabaseTable trait. It also generates access tokens for
authentication (this might need some improvement).

### Client

The client is running as a WASM module in the browser, and is written using the
Seed-rs framework. It handles different pages and components and provides a
graphical UI to the database accessed by the server.

## TO DO

- Authentication
  - [ ] User Group system for controling authenticated routes
  - [ ] Define authenticatied routes in api macro
- Client
  - [ ] Styling
- General
  - [ ] User Comments
  - [ ] User Profile Pictures
  - [ ] User (tagged) votes 
  - [ ] User join Project
  - [ ] Add Task to Project
  - [ ] Define Tasks as tests
  - [ ] Projects/Goals
  - [ ] [SVG Graphs](https://cetra3.github.io/blog/drawing-svg-graphs-rust/) to visualize data
  - [ ] Cloud hosting (AWS? ...)
- Testing
  - [ ] test api macro

# Budget App
### A social network for managing projects

## Development

This project is built using Rust and WebAssembly.

Backend:
- [Rocket](https://rocket.rs/) [server](server/)
- [rql](https://github.com/kaikalii/rql) database

Frontend:
- [Seed](https://seed-rs.org/) Web Client Framework

### Building the client and running the server
```
  cargo make
```

`cargo make` will build the client to [wasm32-unknown-unknown](https://rustwasm.github.io/book/),
build the server and run it. Other [cargo-make](https://github.com/sagiegurari/cargo-make) targets
are defined in the `Makefile.toml` files.

### Client/Server API
The server and client both import the [api](api) crate, which uses
[procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html)
to generate code for an RPC interface between clients and servers (the server
address is still fixed to 0.0.0.0:8000 currently). This interface allows to
easily define functions on the server that can be invoked from the client. Using
this, database queries, authentication and all other client/server communication
is realized.

### Server

The server accepts requests by clients and accesses the database tables
through the [`DatabaseTable`](https://github.com/mankinskin/database-table/blob/master/src/table.rs) trait. It also generates access tokens for
authentication (this might need some improvement).

- [routes](server/src/main.rs)

### Database
- Database
  - [x] REST API
  - [ ] store timestamps
  - [ ] store entry update history
  - [ ] database version history
- Users
  - [ ] Comment content
    allow users reply content to other content
  - [ ] Profile Pictures
  - [ ] tagged voting 
    allow users to tag content with an associated weight
  - [ ] user rating
    rate a user based on the votes and tags on their content
  - [ ] join Project
  - [ ] Add Task to Project
  - [ ] Define Tasks as tests
  - [ ] Projects/Goals
- Tasks
  - [ ] Dependencies
  - [ ] Results (Resources)
  - [ ] related work
- Projects
  - [x] Task tree
- Modelling
  - [ ] Context
    relations between data (when, where, what)
  - [ ] Resources/States
    Model measurable states in the world (data), associated with a context.
    Every data event has a time associated with it. every other data event
    around that time is likely to be contextual. Other relationships like topic,
    user, place, etc. also indicate a contextual relationship.
  - [ ] Relations
    representations can be related and depend on other data 
  - [ ] Goals
    conditions describing states wanted by someone

### Client

The [client](client) is running as a WASM module in the browser, and is written using the
Seed-rs framework. It handles different pages and components and provides a
graphical UI to the database accessed by the server.

- [ ] Styling
- [Root](client/src/root.rs)
  - Pages
    - Home
    - User List
    - User Profile
    - Login
    - Register
    - Project List
    - Project Profile
    - Task Profile
  - Navbar
- Visualization
  - [ ] [SVG Graphs](https://cetra3.github.io/blog/drawing-svg-graphs-rust/) to visualize data

### Client/Server API
  - [RPC calls](api/src/lib.rs)
    - REST handlers for tables
  - [Authentication](api/src/auth.rs)
    - [x] simple api for login/register
    - [ ] User Group system for controling authenticated routes
    - [ ] Define authentication strategy in api macro
    - [ ] use [casbin-rs](https://github.com/casbin/casbin-rs)?
  - [ActivityPub](https://www.w3.org/TR/activitypub/)
    - [ ] implement protocol endpoints

### Natural Language Interpreter
Running on the server, it stores a graph of words. It can read texts and store
them in the graph efficiently, by only storing possible connections between
multiple words. later it can be used to find words to fill gaps in texts, or
generate sentences.

- [Interpreter](interpreter)
  - [x] word graph storing sentences
  - [x] find subgraph
  - [ ] advanced queries (regex)
  - [ ] word/edge occurrence count
  - [ ] word classification
  - [ ] semantic annotation
  - [ ] contextual semantics

## General
- physical implementation
  - [ ] Cloud hosting (AWS? ...)
- Testing
  - [ ] test api macro

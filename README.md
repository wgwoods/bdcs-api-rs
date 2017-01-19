# Composer BDCS API Library and Server

This codebase is the BDCS API server, it handles API requests for project
information, recipes, and image composing.

It depends on the BDCS metadata store generated by the [bdcs import
service](https://github.com/wiggum/bdcs), and can be run directly via `cargo
run` if the system has Rust installed, or via Docker.

## Running bdcs directly on the host

The server requires the nightly version of Rust which is best installed by
following the instructions at https://rustup.rs, and then overriding the
default compiler to use for the bdcs project by running this in the repo
directory:

`rustup override set nightly`

Running it directly on port 4000, using /var/tmp/recipes/ for recipe storage
looks like this:

`cargo run -- --host 0.0.0.0 --port 4000 metadata.db /var/tmp/recipes/`

This will download any required crates, build them, build bdcs and run it.

If you want to use the /api/mock/ service you can point it to a directory of
json mock api files by adding `--mockfiles /path/to/files/`


## Running the API Server in Docker

Build the docker image by running:

`sudo docker build -t wiggum/bdcs-api .`

To run the API it requires access to a copy of the metadata.db created by the
[bdcs import service](https://github.com/wiggum/bdcs) and to a directory of
recipes. The recipes directory is initialized at runtime from the
./examples/recipes/ directory.

Create `~/tmp/mdd/` and copy metadata.db into it, and create an empty
`~/tmp/recipes/` directory. You can then run the API server like this:

`docker run -it --rm -v ~/tmp/mddb/:/mddb/:Z -v ~/tmp/recipes/:/bdcs-recipes/:Z -p 4000:4000 wiggum/bdcs-api`

You can then access the UI at `http://localhost:3000`, try `http://localhost:3000/api/v0/test` to
make sure you get a response like `API v0 test` from the server.

If you want to use a local directory named mock-api for the `/api/mock/`
service you would add this to the commandline (before wiggum/bdcs-api):
`-v ~/tmp/mock-api/:/mockfiles/:Z`

The files in tests/results/v0/ are suitable to use with the `/api/mock/` service.

See the [documentation on the mock api](src/api/mock.rs) for more information.

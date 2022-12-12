# install needed packages
    sudo apt-get install libpq-dev

    cargo install diesel_cli --no-default-features --features postgres

# init database postgres

    createdb calendar

Change .env, if needed.

    diesel migration run

# server
     cargo run (or ``cargo watch -x run``)

Started http server: 127.0.0.1:3000

Then go to http://localhost:3000/.

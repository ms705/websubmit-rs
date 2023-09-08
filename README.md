# websubmit-rs: a simple class submission system

This is a fork for websubmit-rs, a web application for collecting student homework
submissions, written using [Rocket](https://rocket.rs) for a [K9db](https://github.com/brownsys/K9db) backend.

To run it, you need to run a K9db server deployment.
Then you can run the web application, which will automatically connect
to a K9db database `myclass`:
```
websubmit-rs$ cargo run --release -- -i myclass
```
To create and initialize the database, set the `prime` variable in the configuration
file (see below).

The web interface will be served on `localhost:8000`. Note that the
templates included in this repository are very basic; in practice, you
will want to customize the files in `templates`.

By default, the application will read configuration file `sample-config.toml`,
but a real deployment will specify a custom config (`-c myconfig.toml`).
Configuration files are TOML files with the following format:
```
# short class ID (human readable)
class = "CSCI 2390"
# K9db database user
db_user = "root"
# K9db database password
db_password = "password"
# Database address with port
db_addr = "127.0.0.1:10001"
# Backup log file
backup_file = "/tmp/backup.sql"
# list email addresses whose API keys get admin access
admins = ["malte@cs.brown.edu"]
# list email addresses who should receive notification emails
staff = ["malte@cs.brown.edu"]
# custom template directory
template_dir = "/path/to/templates"
# custom resource directory (e.g., for images, CSS, JS)
resource_dir = "/path/to/resources"
# a secret that will be hashed into user's API keys to make them unforgeable
secret = "SECRET"
# whether to send emails (set to false for development)
send_emails = false
# whether to reset the db (set to false for production)
prime = true
```

If you omit `--release`, the web app will produce additional
debugging output.

## Running and Configuring K9db.

1. Fork K9db from https://github.com/brownsys/K9db into some directory, e.g. `/k9db`.
2. Checkout this commit `7c16e705ca0ed63178c6cf7ef46b3450d64b2900`.
3. Build k9db using the instructions in the [wiki](https://github.com/brownsys/K9db/wiki/Requirements%3A-Ubuntu-and-similar-distros), you can use the docker container if desired: [instructions](https://github.com/brownsys/K9db/wiki/Requirements%3A-Using-Docker).
4. Inside the K9db directory, run the following to start the database:
```bash
bazel run //:k9db -- --db_path="<path/to/dir>" --db_name="<DBname>" > log.out 2> error.out &
# For example, this creates a database under /data/csci2390
mkdir -p /k9db_data
bazel run //:k9db -- --db_path="/k9db_data" --db_name="csci2390" > log.out 2> error.out &
```

You can shutdown K9db at any time, and then restart it without losing data by running
the same command above. If you want to start a clean instance of K9db, delete the DB path before
start K9db (you will lose all your data) or use a different db_name.

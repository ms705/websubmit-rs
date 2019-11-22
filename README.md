# websubmit-rs: a simple class submission system

This is a work-in-progress web application for collecting student homework
submissions, written using [Rocket](https://rocket.rs) for a
[Noria](https://pdos.csail.mit.edu/noria) backend.

To run it, you need to run a Noria server deployment (note that this
requires installing Zookeeper on your machine):
```
noria$ cargo run --release --bin noria-server -- --deployment myclass
```
Then you can run the web application, which will automatically connect
to Noria:
```
websubmit-rs$ cargo run --release -- -i myclass
```
The web interface will be served on `localhost:8000`. Note that the
templates included in this repository are very basic; in practice, you
will want to customize the files in `templates`.

The application also supports specifying a custom configuration file
(`-c myconfig.toml`), which is a TOML file with the following format:
```
# short class ID (human readable)
class = "CSCI 2390"
# list of staff email addresses (these users' API keys get admin access)
staff = ["malte@cs.brown.edu"]
# custom template directory
template_dir = "/path/to/templates"
# custom resource directory (e.g., for images, CSS, JS)
resource_dir = "/path/to/resources"
# a secret that will be hashed into user's API keys to make them unforgeable
secret = "SECRET"
# whether to send emails (set to false for development)
send_emails = false
```

If you omit `--release`, both Noria and the web app produce additional
debugging output.


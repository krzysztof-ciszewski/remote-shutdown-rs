Simple remote shutdown server that listens on port 8000 on endpoint `/<secret>/shutdown`, inspired by https://github.com/karpach/remote-shutdown-pc

## Setup
First run `./remote_shutdown` this will create a file in `$XDG_CONFIG_DIR/remote_shutdown/secret`

The default content is `secret`, you can change that to whatever random characters you want. You will have to include the secret when calling shutdown endpoint, for default configuration it will be `127.0.0.1:8000/secret/shutdown`

The default delay is 60s, you can change that by passing query parameter to the endpoint, for example `/secret/shutdown?delay=30`

After the endpoint is called a popup will appear where you can abort the shutdown process.

To change the port, run the binary with `ROCKET_PORT` env variable, for example: `ROCKET_PORT=2137 ./remote_shutdown`
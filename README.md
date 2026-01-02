# lbin

A minimal pastebin made for liminal.cafe

## config

Make a config.toml file in the root directory and fill it with the following:

```toml
[env]
lbin_host="localhost"
lbin_url ="https://root_url_for_hosted_files.com"
lbin_port="8000"
lbin_auth="auth_code_here"
```

After you have your config file, you can then compile it.

```
cargo build --release --config config.toml
```

## usage
```sh
# basic
curl -F "file=@some_file_here" -H "Authorization: Bearer <AUTH_CODE_HERE>" <URL_HERE>
```
```sh
# file expires in 5 minutes
curl -F "file=@some_file_here" -F "time=5" -H "Authorization: Bearer <AUTH_CODE_HERE>" <URL_HERE>
```
Or you can use [lbin-cli](https://codeberg.org/gnp/lbin-cli) for a more intuitive experience.

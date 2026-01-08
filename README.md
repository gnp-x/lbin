# lbin

A minimal pastebin made for liminal.cafe

## config

Make a config.toml file in the root directory and fill it with the following:

```toml
[env]
host="localhost"
url ="https://root_url_for_hosted_files.com"
port="8000"
auth="auth_code_here"
tmp="temp_folder_name_here"
os="oneshot_folder_name_here"
```

After you have your config file, you can then compile it.

```
cargo build --release --config config.toml
```

## usage
Use [lbin-cli](https://git.liminal.cafe/sakura/lbin-cli) to easily upload files, or use [byakuren's lbin-cli](https://web.liminal.cafe/~byakuren/sh/lbin/)

A Solid pod server recipe from Manas project.

This server serves a single pod over filesystem backend, without authentication or access-control.

## Installation

### Binaries
Currently binaries are provided for linux through github releases.

### Through Cargo
```sh
cargo install manas_server_single_fs_noauth
```

Note that it performs entire compilation on your machine.

## usage

```sh
manas_server_single_fs_noauth -c config.toml
```

Example configuration file is provided at `config-template.toml`.

It is required to configure owner webid. Currently Manas project doesn't include an identity provider. You may use one from any of the existing solid-oidc compliant idp. You may have to use local community solid server's idp, or any of the cloud services [listed](https://solidproject.org/users/get-a-pod#get-a-pod-from-a-pod-provider).

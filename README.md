# oci-run: a tool for running OCI containers

This is free and unencumbered software released into the public domain.

`oci-run` is a tool that provides a configurable way to run commands in
disposable OCI containers with user-defined profiles based on the current
directory.

## Usage

Create `~/.config/oci-run/config.yaml` and define some profiles:

```
profiles:
  ~/src/proj1:
    image: alpine
    volumes:
      - "${PWD}:/app"
    workdir: /app
```

Profiles require the following keys:

- `image`: the image used to run containers

Profiles may also optionally define:

- `entrypoint`: true (default) or false to enable the custom entrypoint script
- `env`: map of environment names (keys) and their values to define
- `path-append`: list of directories to add to the end of PATH
- `path-prepend`: list of directories to add to the beginning of PATH
- `setpriv`: true (default) or false to use setpriv in the custom entrypoint
- `user`: unprivileged user name when using the custom entrypoint
- `user-gid`: unprivileged user GID when using the custom entrypoint
- `user-uid`: unprivileged user UID when using the custom entrypoint
- `volumes`: list of volumes to mount in the container
- `workdir`: container working directory

To run a command inside a container, call it with `oci-run` to use the profile
for the current directory:

```
$ cd ~/src/proj1
$ pwd
/home/user/src/proj1
$ oci-run -- pwd
/app
```

The custom entrypoint automatically creates an unprivileged user inside the
container with a UID and GID matching the host user's ID, then drops privileges
before executing the specified command:

```
$ oci-run -- id
uid=1000(user) gid=1000(user) groups=1000(user)
$ oci-run -- grep Cap /proc/self/status
CapInh: 0000000000000000
CapPrm: 0000000000000000
CapEff: 0000000000000000
CapBnd: 0000000000000000
CapAmb: 0000000000000000
```

## Building

oci-run is written in Rust, and can either be built using a container that
provides the Rust toolchain or by
[installing Rust](https://www.rust-lang.org/learn/get-started).

Using the makefile:

```
$ make
```

Using cargo:

```
$ cargo build --release
```

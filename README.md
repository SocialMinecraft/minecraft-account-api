# Template Repo

This repo is the foundation for creating a new microservice in the yabs project.

## Using this repo
1. Create a new repo on github and select this as the template.
2. Adjust the name of this app in the Cargo.toml and the Dockerfile entry line.

## Creating a release

```sh
cargo release patch/minor/major --execute
````

##  Creating a sql migration

```sh
sqlx migrate add
```

## Update sql scripts for release

This needs to be ran and commited to the repo to allow
for the ci/cd to build with sql.

```shell
cargo sqlx prepare
```

## How to change the proto repo
   
1. Clean up old proto
   ```shell
   git rm proto
   rm -rf .git/modules/proto
   git config --remove-section submodule.proto
   ```
2. Add new submodule
   ```shell
   git submodule add ...
   ```
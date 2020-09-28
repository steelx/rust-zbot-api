


### deps
```
#argonautica needs cc bindgen
sudo apt install clang llvm-dev libclang-dev
```


```
docker-compose up -d

```


### Install sqlx cli
```
cargo install --version=0.1.0-beta.1 sqlx-cli
```


### Create DB tables
```
sqlx mig add create_tables

#Generate DB into the specified database_url from .env file
sqlx mig run
```


### Local DB testing
```
user@user-pc:~$ sudo -i -u postgres
postgres@user-pc:~$ psql
psql (9.3.5, server 9.3.6)
Type "help" for help.
```
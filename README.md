


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

#Generate DB into the specified database_url from env
sql mig run
```
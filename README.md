
Frontend:
Flutter (Android) https://github.com/steelx/android-zbot-app

### deps
```
#argonautica needs cc bindgen
sudo apt install clang llvm-dev libclang-dev
```

### Run posgress DB
```
docker-compose up -d
```

### Port already in use error FIX & try again
```
sudo service postgresql stop
```


### Install sqlx cli
```
cargo install --version=0.1.0-beta.1 sqlx-cli
```


### Create DB tables
https://github.com/launchbadge/sqlx/tree/master/sqlx-cli
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

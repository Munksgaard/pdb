# pdb

A very simple attempt at an (in-memory, at least so far) database that has
support for algebraic datatypes.

More information [here](https://munksgaard.me/pdb/lets-build-a-database.html)
and [here](https://munksgaard.me/pdb/create-insert-and-select.html).

## Example usage

In one terminal, start the server:

```
cargo run -- --port 8080
```

In another, start the client:

```
$ cargo run --bin pdbcli -- -d localhost:8080
Welcome to pdbcli!
Connected to localhost:8080!
>> create table user Int
Created

>> insert let x = 4 in x end into user
Inserted 1

>> select from user
[4]

>> insert let f = lambda x -> lambda y -> x in f 42 43 end into user
Inserted 1

>> select from user
[4, 42]
```

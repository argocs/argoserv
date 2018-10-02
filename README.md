### argoserv  
A Gopher server implemented in Rust

### Making a server

This server only serves `index.gph` files in directories, so you must include one for every directory you wish to have in the server.

`index.gph`'s are plain text documents that conform to the Gopher protocol.

The server will replace the following tokens automatically:

`$ADDRESS$` -> The IP address specified at runtime. **Note**: If you plan on making the server public, use your public IP address here. Local IP addresses will, for obvious reasons, not work for anyone outside of your intranet.  
`\t` -> A tab character. Note this is a backslash followed by a `t`. Not an actual tab character.

I recommend using `$ADDRESS$` for local host selectors in your `index.gph`s so you need not rewrite the IP everytime it changes.

See included example site for working examples `index.gph`s.

### Usage

```
$ ./argoserv <ip> [site directory]
argoserv listening on <ip>:70
```

### Example gopher site

This repo comes with an example site that was used while testing. You may run it like so:

```
$ cargo run -- <ip> site
```

### License

MIT. See `LICENSE`

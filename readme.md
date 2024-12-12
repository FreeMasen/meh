# Meh-rs

## lib

For use building Rust applications that consume the Meh API. 

Usage:

```rust
use meh::{Deal, Poll, Response, Video};
static KEY: &str = include_str!("path/to/api/key");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = meh::construct_url(KEY);
    let json = make_network_request();
    let r: Response = serde_json::from_str(&json);
    println!("{}", r.deal.title);
    println!("{}", r.poll.title);
    println!("{}", r.video.title);
}

fn make_network_request(url: &str) -> String {
    // get your json somehow
    // ..snip..
}
```

## `meh-cli`

```console
Usage: meh-cli <COMMAND>

Commands:
  watch  
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help 
```

### `watch`


```
Usage: meh-cli watch [OPTIONS] <API_KEY>

Arguments:
  <API_KEY>  

Options:
  -p, --progress             
  -i, --interval <INTERVAL>  [default: 60]
  -h, --help                 Print help
```

```console
▒ Bag of Crap (5.00)
▒ https://meh.com/deals/bag-of-crap
▒ checking again in 12 hours at 10:00 pm
```

The first line is the title and the price, in USD. If there is more than one option, all prices will be included
in the parentheses. The next line will be a link directly to the item (your terminal might not treat it like a hyperlink though). The third line will let you know when the next update is, when it is more than an hour away, it will update each hour, in the last hour, it will update each minute.

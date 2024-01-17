## Some notes:

Architecture Diagram: https://excalidraw.com/#room=5be369b044c8e2e4ae94,rekyHKkC_I2j4rcKaa9hGA

This example will stream binance data and write it to files in the `data` folder (might need to create it):  
[`barter-data-rs/examples/order_books_l2_streams.rs`:](https://github.com/sector-fi/barter-mono/blob/feat/refactor-orderbook/barter-data-rs/examples/order_books_l2_streams.rs#:~:text=order_books_l2_refactor.rs-,order_books_l2_streams,-.rs)
```
cargo run -p barter-data --example order_books_l2_streams
```

Once you have some data files, you can use them here to run a backtest (WIP)  
[`barter-rs/examples/engine_with_historic_book.rs`
](https://github.com/sector-fi/barter-mono/blob/feat/refactor-orderbook/barter-data-rs/examples/order_books_l2_refactor.rs#:~:text=engine_with_historic_book)
```
cargo run -p barter --example engine_with_historic_book
```

This is where I strated pulling some guts to the serface in hopes of simplifying the architecture a little bit  
[`barter-data-rs/examples/order_books_l2_refactor.rs`
](https://github.com/sector-fi/barter-mono/blob/feat/refactor-orderbook/barter-data-rs/examples/order_books_l2_refactor.rs#:~:text=order_books_l1_streams_multi_exchange.rs-,order_books_l2_refactor,-.rs)

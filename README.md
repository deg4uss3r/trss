# trss

`trss` is a Terminal User Interface (TUI) RSS reader. In other words, read your favorite website's posts directly from your terminal!

Place websites in a config file which is created at the $XDG location (using [confy](https://docs.rs/confy/latest/confy/)) on your system.

The config is stored normally in `~/.config/trss/trss.toml` and looks like:

```toml
subscriptions = ["https://everythingchanges.us/feed.xml", "https://charity.wtf/feed/"]
```

To show the help hit `h` in any view mode, the basics are:

```text
ENTER - Select website or article
Arrow Keys - Navigate the UI/Scroll in the article 
ESC - Return to previous panel
Q - Same as ESC, return to the previous panel
H - Show help
```

## TODO

- [ ] Edit config in UI
- [ ] Keep track of "read" articles

## License

See `LICENSE.md`.

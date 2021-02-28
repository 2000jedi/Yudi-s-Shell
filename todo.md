# todo-list
## Cuurent Works

- [x] fix `~` for path
- [x] multi-line command (`\` symbol)
- [x] putting process in background (`&` ending)
- [x] background subprocesses listening and recycling
- [x] pass signals to children (`Ctrl-C` and `Ctrl-Z`)
- [x] block signals to background processes.  
    - [ ] FIXME: this can only be fixed by libc::setpgid, which is not supported in crate `subprocess`.
- [ ] `bg` and `fg` command

## Future Works
- [ ] Support REPL with GNU/readline
- [ ] add `PATH` and other environment variables
- [ ] `export` command
- [ ] parse variable (`$` symbol)
- [ ] `if` and `for` command
- [ ] load `/etc/profile` and `~/.rsh_profile` on startup
- [ ] load `~/.rshrc` during REPL

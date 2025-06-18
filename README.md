
### hyprzoom-rs
`hyprzoom-rs` is a tool that uses global keybinds in order to allow zooming in on the cursor as hyprctl only lets you set to static value it doesn't let u zoom x amount every time keybind is triggered (at least i couldn't find a way)

### build instructions:

    git clone https://github.com/awaprim/hyprzoom-rs.git
    cd hyprzoom-rs
    cargo build --release

### usage:
hyprland.conf:

    bind = SUPER, mouse_down, global, :zoom
    bind = SUPER, mouse_up, global, :unzoom 

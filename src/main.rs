use futures_util::StreamExt;
use std::{env, io::Write, os::unix::net::UnixStream};

use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};

unsafe extern "C" {
    fn getuid() -> i32;
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let keybind = create_shortcuts().await;
    let mut zoom_rx = keybind.receive_activated().await.unwrap();
    let mut current_zoom = 1.0;
    let xdg = env::var("XDG_RUNTIME_DIR");
    let xdg = if xdg.is_ok() {
        xdg.unwrap() + "/hypr"
    } else {
        let uid = unsafe { getuid() };
        format!("/run/user/{uid}/hypr")
    };
    let instance_sig = env::var("HYPRLAND_INSTANCE_SIGNATURE");
    let instance_sig = if instance_sig.is_ok() {
        instance_sig.unwrap()
    } else {
        panic!("instance signature was not found!");
    };
    let socket_path = format!("{xdg}/{instance_sig}/.socket.sock");

    loop {
        let resp = zoom_rx.next().await.unwrap();
        let id = resp.shortcut_id();
        match id {
            "zoom" => {
                current_zoom = f64::max(1.0, current_zoom * 1.2);
            }
            "unzoom" => {
                current_zoom = f64::max(1.0, current_zoom / 1.2);
            }
            _ => {
                break;
            }
        }
        let mut socket = UnixStream::connect(&socket_path).unwrap();
        socket
            .write_all(format!("keyword cursor:zoom_factor {current_zoom}").as_bytes())
            .unwrap();
    }
}

async fn create_shortcuts() -> GlobalShortcuts<'static> {
    let shortcut = GlobalShortcuts::new().await.unwrap();
    let session = shortcut.create_session().await.unwrap();
    let short = NewShortcut::new("zoom", "increases zoom");
    let bind = shortcut
        .bind_shortcuts(&session, &[short], None)
        .await
        .unwrap();
    bind.response().unwrap();
    let short = NewShortcut::new("unzoom", "decreases zoom");
    let bind = shortcut
        .bind_shortcuts(&session, &[short], None)
        .await
        .unwrap();
    bind.response().unwrap();
    shortcut
}

use futures_util::StreamExt;
use std::{env, io::Write, os::unix::net::UnixStream, path::PathBuf};
use tokio::{fs::File, io::AsyncWriteExt};

use ashpd::{
    AppID, WindowIdentifier,
    desktop::{
        CreateSessionOptions,
        global_shortcuts::{BindShortcutsOptions, GlobalShortcuts, NewShortcut},
    },
    register_host_app,
};

unsafe extern "C" {
    fn getuid() -> i32;
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    {
        // no error handling needed idc about it that much...
        let home = env!("HOME");
        let mut path = PathBuf::from(home);
        path.push(".local");
        path.push("share");
        path.push("applications");
        path.push("com.fisch.Hyprzoom.desktop");
        let mut file = File::create(path).await.unwrap();
        file.write_all(include_bytes!("../desktop_file")).await.unwrap();
    }

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
        socket.write_all(format!("keyword cursor:zoom_factor {current_zoom}").as_bytes()).unwrap();
    }
}

async fn create_shortcuts() -> GlobalShortcuts {
    let app_id = AppID::try_from("com.fisch.Hyprzoom").unwrap();
    register_host_app(app_id).await.unwrap();
    let options = CreateSessionOptions::default();
    let shortcut = GlobalShortcuts::new().await.unwrap();
    let session = shortcut.create_session(options).await.unwrap();
    let options = BindShortcutsOptions::default();
    let short = NewShortcut::new("zoom", "increases zoom");
    let bind = shortcut.bind_shortcuts(&session, &[short], None, options).await.unwrap();
    bind.response().unwrap();
    let options = BindShortcutsOptions::default();
    let short = NewShortcut::new("unzoom", "decreases zoom");
    let bind = shortcut.bind_shortcuts(&session, &[short], None, options).await.unwrap();
    bind.response().unwrap();
    shortcut
}

// Ayuz - Unofficial Control Center for Asus Laptops
// Copyright (C) 2026 Guido Philipp
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see https://www.gnu.org/licenses/.

use gtk4::glib;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixStream;
use tracing::{info, warn};

pub fn start() {
    let socket_path = "/var/run/acpid.socket";

    tokio::spawn(async move {
        let stream = match UnixStream::connect(socket_path).await {
            Ok(s) => s,
            Err(e) => {
                warn!(
                    "Fn-Lock WMI listener: failed to connect to {}: {}",
                    socket_path, e
                );
                return;
            }
        };

        info!("Fn-Lock WMI listener active on {}", socket_path);
        let mut reader = BufReader::new(stream).lines();
        let state = Arc::new(AtomicBool::new(true));

        while let Ok(Some(line)) = reader.next_line().await {
            // Check for Asus Fn-Lock WMI event: wmi PNP0C14:00 000000ff
            if line.contains("PNP0C14:00") && line.contains("000000ff") {
                let current = state.fetch_xor(true, Ordering::SeqCst);
                let new_state = !current;

                glib::idle_add_once(move || {
                    crate::components::keyboard::fn_osd::show(new_state);
                });
            }
        }
    });
}

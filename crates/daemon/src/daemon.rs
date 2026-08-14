use std::{ffi::OsString, fs::OpenOptions, io::Write, thread, time::Duration};

use crate::niri::{NiriConnector, NiriListener};

pub struct Daemon {
    niri_listener: NiriListener
}

impl Daemon {
    pub fn new(niri_socket_path: OsString) -> Self {
        let niri_connector = 
            NiriConnector::new(niri_socket_path.into());

        Self {
            niri_listener: NiriListener::new(niri_connector)
        }
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        self.niri_listener
            .run()
            .await?;
        Ok(())
    }

    pub fn heartbeat(&self) -> Result<Self, anyhow::Error> {
        loop {
            let mut file = OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open("/tmp/niqol.log")?;
            writeln!(file, "niqol heartbeat...")?;
            thread::sleep(Duration::from_secs(1));
        }
    }
}

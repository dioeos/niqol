use std::{
    fs::OpenOptions,
    io::Write,
    thread,
    time::Duration
};

fn main() -> std::io::Result<()> {
    loop {
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open("/tmp/niqol.log")?;
        writeln!(file, "niqol hearthbeat...")?;
        thread::sleep(Duration::from_secs(1));
    }
}

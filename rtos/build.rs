use {
    chrono::{Datelike, Local, Timelike},
    std::{collections::HashMap, fs, io, path::PathBuf, process::Command},
};

const GEN_DIR: &str = "gen";
const CONSTS_FILE: &str = "consts.rs";

fn main() {
    out_dir().unwrap();
    // Rows
    let mut rows = HashMap::<&'static str, (&'static str, String)>::new();

    // Time
    rows.insert(
        "RUSTC_VERSION",
        ("&str", rustc_version::version().unwrap().to_string()),
    );

    let offset = chrono::FixedOffset::east_opt(0).unwrap();
    let now = Local::now().with_timezone(&offset);

    // let now = Local::now();
    rows.insert("COMPILE_TIME_YEAR", ("i32", now.year().to_string()));
    rows.insert("COMPILE_TIME_MONTH", ("u32", now.month().to_string()));
    rows.insert("COMPILE_TIME_DAY", ("u32", now.day().to_string()));
    rows.insert("COMPILE_TIME_HOUR", ("u32", now.hour().to_string()));
    rows.insert("COMPILE_TIME_MINUTE", ("u32", now.minute().to_string()));
    rows.insert("COMPILE_TIME_SECOND", ("u32", now.second().to_string()));

    // Git version
    let git_desc = Command::new("git")
        .args(["describe", "--all", "--tags", "--dirty", "--long"])
        .output()
        .unwrap();
    rows.insert(
        "GIT_DESCRIBE",
        (
            "&str",
            String::from_utf8_lossy(&git_desc.stdout).to_string(),
        ),
    );

    let mut contents = Vec::<String>::with_capacity(rows.len());
    for (n, (t, v)) in rows {
        if t == "&'static str" || t == "&str" {
            contents.push(format!(
                "pub const {}: {} = \"{}\";",
                n.to_uppercase(),
                t,
                v.trim()
            ));
        } else {
            contents.push(format!(
                "pub const {}: {} = {};",
                n.to_uppercase(),
                t,
                v.trim()
            ));
        }
    }

    fs::write(
        PathBuf::from(GEN_DIR).join(CONSTS_FILE),
        contents.join("\n") + "\n",
    )
    .unwrap();
}

fn out_dir() -> Result<(), io::Error> {
    let path = PathBuf::from(GEN_DIR);
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    fs::create_dir_all(&path)
}

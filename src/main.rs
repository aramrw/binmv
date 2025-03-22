use color_eyre::Result;
use color_print::cprintln;
use std::{
    env::{self, current_dir},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use thiserror::Error;

#[derive(Error, Debug)]
enum CliParseError {
    #[error("required argument was not passed: --{arg}")]
    MissingRequiredArg { arg: String },
    #[error("argument passed without a value: --{arg} <{expected}>")]
    MissingArgumentValue { arg: String, expected: String },
    #[error("--{arg} takes no value")]
    BoolArg { arg: String },
}

#[derive(Error, Debug)]
enum InitCheckError {
    #[error("missing Cargo.toml file in current directory: {}", dir.display())]
    MissingCargoToml { dir: PathBuf },
    #[error("release directory not found in target folder: {}", dir.display())]
    MissingReleaseDir { dir: PathBuf },
    #[error("executable binary not found in release directory: {}", bin_path.display())]
    MissingReleaseBinary { bin_path: PathBuf },
    #[error("shhhh...")]
    Fail,
}

#[derive(Default)]
struct Cli {
    /// --name
    name: String,
    /// --build
    build: bool,
}

fn main() -> Result<()> {
    let cargotoml = Path::new("./Cargo.toml");
    let releasedir = PathBuf::from("./target").join("release");
    let cdir = current_dir()?;
    let cli = Cli::parse()?;

    if !cargotoml.exists() {
        return Err(InitCheckError::MissingCargoToml { dir: cdir }.into());
    }

    if cli.build {
        cprintln!("--build arg, running <b>cargo build --release</>");
        let mut child = Command::new("cargo")
            .args(["build", "--release"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(stdout) = child.stdout.take() {
            let stdout_reader = BufReader::new(stdout);
            for line in stdout_reader.lines() {
                match line {
                    Ok(line) => println!("{}", line),
                    Err(e) => eprintln!("Error reading stdout: {}", e),
                }
            }
        }

        if let Some(stderr) = child.stderr.take() {
            let stderr_reader = BufReader::new(stderr);
            for line in stderr_reader.lines() {
                match line {
                    Ok(line) => eprintln!("{}", line),
                    Err(e) => eprintln!("Error reading stderr: {}", e),
                }
            }
        }

        let status = child.wait()?;
    }

    if !releasedir.exists() {
        return Err(InitCheckError::MissingReleaseDir {
            dir: cdir.join("target"),
        }
        .into());
    }

    let os = env::consts::OS;
    let version = env::var("CARGO_PKG_VERSION").unwrap_or("?.?.?".into());
    let name = env::var("CARGO_PKG_NAME").unwrap();
    let release_bin_path = releasedir.join(name);
    if !release_bin_path.exists() {
        return Err(InitCheckError::MissingReleaseBinary {
            bin_path: release_bin_path,
        }
        .into());
    }
    let new_bin_name = format!("{}-{}-{}", cli.name, os, version);
    std::fs::rename(&release_bin_path, releasedir.join(&new_bin_name))?;
    cprintln!(
        "\n<g>SUCCESS</> renamed <r>{}</> to: <b>{}</b>",
        release_bin_path.display(),
        releasedir.join(&new_bin_name).display()
    );
    Ok(())
}

fn missing_arg_err(arg: &str, exp: &str) -> CliParseError {
    CliParseError::MissingArgumentValue {
        arg: arg.into(),
        expected: exp.into(),
    }
}

impl Cli {
    fn parse() -> Result<Self, CliParseError> {
        let args = env::args();
        let args: Vec<String> = args.into_iter().collect();
        let mut cli = Cli::default();
        for (i, arg) in args.iter().enumerate() {
            match arg.as_str() {
                "--name" => match args.get(i + 1) {
                    Some(next) => {
                        if next.starts_with("-") {
                            return Err(missing_arg_err("name", "TEXT"));
                        }
                        cli.name = next.clone();
                    }
                    None => {
                        return Err(missing_arg_err("name", "TEXT"));
                    }
                },
                "--build" => {
                    if let Some(next) = args.get(i + 1) {
                        if !next.starts_with("-") {
                            return Err(CliParseError::BoolArg {
                                arg: "build".into(),
                            });
                        }
                        cli.build = true;
                    }
                }
                _ => {}
            }
        }
        if cli.name.is_empty() {
            return Err(CliParseError::MissingRequiredArg { arg: "name".into() });
        }
        Ok(cli)
    }
}

// fn do_next(i: usize, args: Vec<String>) {
//     match args.get(i + 1) {
//         Some(next) => {
//             if next.starts_with("-") {
//                 return Err(missing_argument_err("name", "TEXT"));
//             }
//             cli.name = next.clone();
//         }
//         None => {
//             return Err(missing_arg_err("name", "TEXT"));
//         }
//     }
// }

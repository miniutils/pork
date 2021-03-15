use std::{env, path::Path, process::Command};

mod config;
mod errors;
mod password;

pub use errors::{error, RadError, Result};
use libc::setuid;
use password::check_password;

fn main() -> Result<()> {

    let config_file = "/etc/rad.toml";

    let args = std::env::args().skip(1).collect::<Vec<_>>();
  
    let usage = format!("Usage: {} command ARGS...", env!("CARGO_PKG_NAME"));

    if args.len() < 1 {
        return Err(error(usage));
    }

    if args[0] == "-h" || args[0] == "--help" {
        println!("{} - execute commands as administrator", env!("CARGO_PKG_NAME"));
        println!("{}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("{}", usage);
        println!();
        println!("OPTIONS:");
        println!("\tcommand      \tThe command to run as root.");
        println!("\targs...      \tThe arguments to pass to the previously mentionned command.");
        println!("\t-h, --help   \tDisplays this message.");
        println!("\t-v, --version\tDisplays version information.");
        return Ok(());
    } else if args[0] == "-v" || args[0] == "--version" {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(())
    }
    
    let command = &args[0];

    if !Path::new(config_file).exists() {
        Err(error(format!(
            "Cannot find file `{}`. Consider creating it and adding content to it to use {}",
            config_file,
            env!("CARGO_PKG_NAME")
        )))
    } else {
        let user = env::var("USER").unwrap();
        let (authorized, no_password) =
            config::can_run_program(command, &user, config_file)?;

        if !authorized {
            return Err(error("You are not authorized to perform this !"));
        }

        if !no_password {
            let mut pass =
                rpassword::prompt_password_stdout(&format!("[rad] Password for {}: ", user))
                    .unwrap();
            let mut counter = 1;
            while !check_password(&user, &pass)? && counter < 3 {
                eprintln!("Authentication failed, please retry.");
                counter += 1;

                pass = rpassword::prompt_password_stdout(&format!("[rad] Password for {}: ", user))
                    .unwrap();
            }

            if counter >= 3 {
                return Err(error("3 invalid password attempts. Aborting."));
            }
        }

        unsafe {
            if setuid(0) != 0 {
                return Err(error("Failed to change user id."));
            }
        }

        let arguments = if args.len() > 1 {
            args[1..].to_vec()
        } else {
            vec![]
        };

        Command::new(&command)
            .args(&arguments)
            .status()?;

        Ok(())
    }
}
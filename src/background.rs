use anyhow::{Result, anyhow};
use std::{env, process, process::Command};

#[cfg(unix)]
pub fn spawn_cache_updater(username: &str) -> Result<()> {
    let exe = env::current_exe()?;

    unsafe {
        match libc::fork() {
            -1 => return Err(anyhow!("Fork failed")),
            0 => {
                if libc::setsid() == -1 {
                    process::exit(1);
                }

                match libc::fork() {
                    -1 => process::exit(1),
                    0 => {
                        let _ = std::env::set_current_dir("/");

                        libc::close(0);
                        libc::close(1);
                        libc::close(2);

                        libc::open(c"/dev/null".as_ptr(), libc::O_RDWR);
                        libc::dup(0);
                        libc::dup(0);

                        let status = Command::new(exe)
                            .arg("--update-cache")
                            .arg(username)
                            .status();

                        match status {
                            Ok(_) => process::exit(0),
                            Err(_) => process::exit(1),
                        }
                    }
                    _ => {
                        process::exit(0);
                    }
                }
            }
            _ => {
                let mut status: i32 = 0;
                libc::waitpid(-1, &mut status, 0);
            }
        }
    }

    Ok(())
}

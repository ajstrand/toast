use color_eyre::eyre::{eyre, Result};
use duct::cmd;
use indicatif::ProgressBar;
use std::{
    io::{prelude::*, BufReader},
    path::{Path, PathBuf},
    sync::Arc,
};
use tracing::instrument;

#[instrument]
pub fn render_to_html(
    dir_of_input_files: String,
    output_dir: String,
    filepaths: Vec<String>,
    npm_bin_dir: PathBuf,
    active_pb: Arc<ProgressBar>,
) -> Result<()> {
    let bin = npm_bin_dir.join("toast-render");
    let bin_str = bin
        .to_str()
        .ok_or_else(|| eyre!("failed to make npm bin into str"))?;
    let mut args: Vec<String> = vec![
        "--unhandled-rejections".to_owned(),
        "strict".to_owned(),
        "--loader".to_owned(),
        "toast/src/loader.mjs".to_owned(),
        bin_str.to_owned(),
        dir_of_input_files,
        output_dir,
    ];
    args.extend(filepaths.iter().cloned());
    let output = cmd("node", args).stderr_to_stdout();
    run_cmd("sourceData", output, active_pb)?;

    Ok(())
}

#[instrument]
pub async fn source_data(
    toast_js_file: &Path,
    npm_bin_dir: PathBuf,
    active_pb: Arc<ProgressBar>,
) -> Result<()> {
    // not a guarantee that toast.js will exist when node
    // goes to look for it: just a sanity check to not
    // execute Command if we don't need to
    if toast_js_file.exists() {
        let bin = npm_bin_dir.join("toast-source-data");
        let bin_str = bin
            .to_str()
            .ok_or_else(|| eyre!("failed to make npm bin into str"))?;
        let output = cmd!(
            "node",
            "--unhandled-rejections",
            "strict",
            "--loader",
            "toast/src/loader.mjs",
            bin_str,
            "/var/tmp/toaster.sock",
            &toast_js_file
                .to_str()
                .ok_or_else(|| eyre!("failed to make toast_js_file into str"))?
        )
        .stderr_to_stdout();

        run_cmd("sourceData", output, active_pb)?;
        Ok(())
    } else {
        // toast file doesn't exist
        // skip running sourceData
        Ok(())
    }
}

fn run_cmd(
    subcommand_name: &str,
    command: duct::Expression,
    active_pb: Arc<ProgressBar>,
) -> Result<()> {
    if let Ok(reader) = command.reader() {
        let reader = Arc::new(reader);
        let thread_reader = reader.clone();
        let child = std::thread::spawn(move || -> std::io::Result<()> {
            let lines = BufReader::new(&*thread_reader).lines();
            for (i, line_result) in lines.enumerate() {
                match line_result {
                    Ok(line) => {
                        // this magic number pulls off the warning
                        if i > 1 {
                            // if the progress bars are hidden, so is the
                            // output from the pb.println function
                            // so we use the println macro instead
                            if active_pb.is_hidden() {
                                println!("{}", line)
                            } else {
                                active_pb.println(line);
                            }
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            Ok(())
        });
        // wait for the process to stop running
        while let Ok(None) = &reader.try_wait() {}
        // wait for thread with stderr/stdout logging from the node
        // process to complete
        let _ = child.join();
        // if the process ended in error, this will return
        match &reader.try_wait()? {
            None => {
                // should never happen because we're while-let'ing above
                panic!("{} reader returned None while still running. This is an unexpected error please report it on github.", subcommand_name)
            }
            Some(output_status) => {
                if output_status.status.success() {
                    Ok(())
                } else if let Some(code) = output_status.status.code() {
                    Err(eyre!(
                        "{} node process exited with code {}",
                        subcommand_name,
                        code
                    ))
                } else {
                    panic!("Should never reach here: 155");
                }
            }
        }
    } else {
        Err(eyre!("{} node process didn't start", subcommand_name))
    }
}

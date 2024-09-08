mod check;

use check::check_subcommand;

use anyhow::Result;

use std::{env::current_dir, path::PathBuf};

use clap::{command, value_parser, Arg, Command};

fn main() -> Result<()> {
    let matches = command!()
        .subcommand(
            Command::new("check").about("Check the project").arg(
                Arg::new("project")
                    .short('p')
                    .long("project")
                    .required(false)
                    .value_parser(value_parser!(PathBuf)),
            ),
        )
        .arg_required_else_help(true)
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("check") {
        let current_dir = current_dir()?;

        let project_argument = matches
            .get_one::<PathBuf>("project")
            .unwrap_or(&current_dir);

        check_subcommand(project_argument.to_path_buf())?;

        return Ok(());
    }

    println!("Unknown subcommand");

    Ok(())
}

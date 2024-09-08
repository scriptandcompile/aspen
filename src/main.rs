use anyhow::Result;

use std::path::Path;
use std::{env::current_dir, path::PathBuf};

use clap::{command, value_parser, Arg, Command};
use vb6parse::parsers::{VB6ClassFile, VB6FormFile, VB6ModuleFile, VB6Project};
use walkdir::WalkDir;

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

fn check_subcommand(project_path_argument: PathBuf) -> Result<()> {
    if !project_path_argument.exists() {
        println!("No project file found at '{:?}'", project_path_argument);
        return Ok(());
    }

    if project_path_argument.is_dir() {
        let search_path = project_path_argument.to_str().unwrap();
        let walker = WalkDir::new(search_path).into_iter();

        println!("Searching '{}' for .vbp project files.", search_path);

        let found_projects: Vec<_> = walker
            .into_iter()
            .filter(|entry| is_project_file(entry))
            .collect();
        let mut error_count = 0;
        let project_count = found_projects.len();

        for project_path in &found_projects {
            if project_path.is_err() {
                println!(
                    "Failed to load project: {}",
                    project_path.as_ref().err().unwrap()
                );
                error_count += 1;
                continue;
            }

            let project_path = project_path.as_ref().unwrap().path();

            check_project(project_path)?;
        }

        println!(
            "{} Projects checked with {} errors",
            project_count, error_count
        );
    } else {
        let project_path = project_path_argument.as_path();
        check_project(project_path)?;
        //println!("Project is not a directory: {:?}", project_argument);
    }

    Ok(())
}

fn is_project_file(entry: &Result<walkdir::DirEntry, walkdir::Error>) -> bool {
    if entry.is_err() {
        return false;
    }

    let entry = entry.as_ref().unwrap();
    entry.path().extension() == Some("vbp".as_ref())
}

fn check_project(project_path: &Path) -> Result<()> {
    let project_contents = std::fs::read(project_path).unwrap();
    let mut _error_count = 0;

    let project_file_name = std::path::Path::new(project_path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    let project = VB6Project::parse(project_file_name, project_contents.as_slice());

    if project.is_err() {
        println!(
            "Failed to load project '{}'\r\n{}",
            project_path.to_str().unwrap(),
            project.err().unwrap()
        );
        return Ok(());
    }

    let project = project.unwrap();

    //remove filename from path
    let project_directory = std::path::Path::new(project_path).parent().unwrap();

    for class_reference in project.classes {
        let class_path = project_directory.join(&class_reference.path.to_string());

        if std::fs::metadata(&class_path).is_err() {
            println!(
                "{} | Class not found: {}",
                project_path.to_str().unwrap(),
                class_path.to_str().unwrap()
            );
            _error_count += 1;
            continue;
        }

        let file_name = class_path.file_name().unwrap().to_str().unwrap();
        let class_contents = std::fs::read(&class_path).unwrap();
        let class = VB6ClassFile::parse(file_name.to_owned(), &mut class_contents.as_slice());

        if class.is_err() {
            println!(
                "{} | Failed to load class '{}' | load error: {}",
                project_path.to_str().unwrap(),
                file_name,
                class.err().unwrap()
            );
            _error_count += 1;
            continue;
        }

        let _class = class.unwrap();
    }

    for module_reference in project.modules {
        let module_path = project_directory.join(&module_reference.path.to_string());

        if std::fs::metadata(&module_path).is_err() {
            println!(
                "{} | Module not found: {}",
                project_path.to_str().unwrap(),
                module_path.to_str().unwrap()
            );
            _error_count += 1;
            continue;
        }

        let file_name = module_path.file_name().unwrap().to_str().unwrap();
        let module_contents = std::fs::read(&module_path).unwrap();
        let module = VB6ModuleFile::parse(file_name.to_owned(), &module_contents);

        if module.is_err() {
            println!(
                "{} | Failed to load module '{}' load error: {}",
                project_path.to_str().unwrap(),
                file_name,
                module.err().unwrap()
            );
            _error_count += 1;
            continue;
        }

        let _module = module.unwrap();
    }

    for form_reference in project.forms {
        let form_path = project_directory.join(&form_reference.to_string());

        if std::fs::metadata(&form_path).is_err() {
            println!(
                "{} | Form not found: {}",
                project_path.to_str().unwrap(),
                form_path.to_str().unwrap()
            );
            _error_count += 1;
            continue;
        }

        let file_name = form_path.file_name().unwrap().to_str().unwrap();
        let form_contents = std::fs::read(&form_path).unwrap();
        let form = VB6FormFile::parse(file_name.to_owned(), &mut form_contents.as_slice());

        if form.is_err() {
            println!(
                "{} | Failed to load form '{}' load error: {}",
                project_path.to_str().unwrap(),
                file_name,
                form.err().unwrap()
            );
            _error_count += 1;
            continue;
        }

        let _form = form.unwrap();
    }

    Ok(())
}

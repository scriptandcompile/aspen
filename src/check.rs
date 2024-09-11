use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use vb6parse::parsers::VB6ProjectReference;
use walkdir::WalkDir;

use vb6parse::parsers::{VB6ClassFile, VB6FormFile, VB6ModuleFile, VB6Project};

pub fn check_subcommand(project_path_argument: PathBuf) -> Result<()> {
    if !project_path_argument.exists() {
        println!("No project file found at '{:?}'", project_path_argument);
        return Ok(());
    }

    let mut error_count = 0;
    let mut project_count = 1;
    if project_path_argument.is_dir() {
        let search_path = project_path_argument.to_str().unwrap();
        let walker = WalkDir::new(search_path).into_iter();

        println!("Searching '{}' for .vbp project files.", search_path);

        let found_projects: Vec<_> = walker
            .into_iter()
            .filter(|entry| is_project_file(entry))
            .collect();

        project_count = found_projects.len();

        for project_path in &found_projects {
            if project_path.is_err() {
                println!("Failed to load {}:", project_path.as_ref().err().unwrap());
                error_count += 1;
                continue;
            }

            let project_path = project_path.as_ref().unwrap().path();

            error_count += check_project(project_path)?;
        }
    } else {
        let project_path = project_path_argument.as_path();
        error_count += check_project(project_path)?;
    }

    if project_count == 1 {
        if error_count == 0 {
            println!("No errors found in project.");
        } else {
            println!("{} errors found in project.", error_count);
        }
    } else {
        if error_count == 0 {
            println!("No errors found in {} projects.", error_count);
        } else {
            println!(
                "{} errors found in {} projects.",
                error_count, project_count
            );
        }
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

fn join_parent_project_path(parent_project_path: &Path, file_path: &str) -> PathBuf {
    let path = PathBuf::from(parent_project_path);

    if cfg!(target_os = "windows") {
        path.join(file_path)
    } else {
        path.join(file_path.replace("\\", "/"))
    }
}

// TODO: Eventually we should be returning an object that contains the errors and the project information.
// This will allow us to display the errors in a more structured way.
// For now we just print the errors to the console and return the error count.

fn check_project(project_path: &Path) -> Result<u32> {
    let project_contents = std::fs::read(project_path).unwrap();
    let mut error_count = 0;

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
        return Ok(error_count);
    }

    let project = project.unwrap();

    //remove filename from path
    let project_directory = std::path::Path::new(project_path).parent().unwrap();

    for reference in project.get_project_references() {
        match reference {
            VB6ProjectReference::Project { path } => {
                let reference_path = join_parent_project_path(project_directory, &path.to_string());
                if std::fs::metadata(&reference_path).is_err() {
                    println!(
                        "{} | Sub-Project Reference not found: {}",
                        project_path.to_str().unwrap(),
                        reference_path.to_str().unwrap()
                    );
                    error_count += 1;
                }
            }
            _ => unreachable!(),
        }
    }

    for class_reference in project.classes {
        let class_path =
            join_parent_project_path(project_directory, &class_reference.path.to_string());

        if std::fs::metadata(&class_path).is_err() {
            println!(
                "{} | Class not found: {}",
                project_path.to_str().unwrap(),
                class_path.to_str().unwrap()
            );
            error_count += 1;
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
            error_count += 1;
            continue;
        }

        let _class = class.unwrap();
    }

    for module_reference in project.modules {
        let module_path =
            join_parent_project_path(project_directory, &module_reference.path.to_string());

        if std::fs::metadata(&module_path).is_err() {
            println!(
                "{} | Module not found: {}",
                project_path.to_str().unwrap(),
                module_path.to_str().unwrap()
            );
            error_count += 1;
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
            error_count += 1;
            continue;
        }

        let _module = module.unwrap();
    }

    for form_reference in project.forms {
        let form_path = join_parent_project_path(project_directory, &form_reference.to_string());

        if std::fs::metadata(&form_path).is_err() {
            println!(
                "{} | Form not found: {}",
                project_path.to_str().unwrap(),
                form_path.to_str().unwrap()
            );
            error_count += 1;
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
            error_count += 1;
            continue;
        }

        let _form = form.unwrap();
    }

    Ok(error_count)
}

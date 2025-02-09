use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub fn discover( working_directory: &Path, verbose: bool ) -> Result<Vec<PathBuf>,std::io::Error> {

    if verbose {
        println!( "scanning {working_directory:?} ..." );
    }

    let mut entries: Vec<(PathBuf,String)> = Vec::new();

    let mut ignore_patterns: Option<BTreeSet<String>> = None;

    for entry in fs::read_dir( &working_directory )? {

        let entry = entry?;

        let entry_path = entry.path();

        let Some( entry ) = entry_path.file_name() else {
            continue;
        };
        let Some( entry ) = entry.to_str() else {
            continue;
        };

        if entry_path.is_dir() {

            if entry.eq( ".git" ) {
                return Ok( vec![ working_directory.to_owned() ] );
            } else {
                entries.push( ( entry_path.clone(), entry.to_owned() ) );
            }
        }

        if entry_path.is_file() && entry.eq( ".reposcanignore" ) {
            ignore_patterns = Some(
                fs::read_to_string( entry_path )?.lines()
                    .map(
                        | line |
                        line.to_owned()
                    ).collect()
            )
        }
    }

    // Potentially filter entries.
    let entries: Vec<_> = match ignore_patterns {
        Some( ignore_patterns ) =>
            entries.into_iter()
                .filter_map(
                    | ( entry_path, entry ) |
                    if !ignore_patterns.contains( &entry ) {
                        Some( entry_path.clone() )
                    } else {
                        None
                    }
                ).collect(),
        None =>
            entries.into_iter()
                .map(
                    | ( entry_path, _ ) |
                    entry_path
                ).collect(),
    };

    let mut repositories = Vec::new();

    for entry_path in entries {

        if entry_path.is_dir() {
            repositories.append(
                &mut discover( &entry_path, verbose )?
            );
        }
    }

    Ok( repositories )
}

pub fn load_known_repositories( repositories_file: &Path ) -> Result<BTreeSet<String>,std::io::Error> {

    let mut repositories: BTreeSet<String> = BTreeSet::new();

    let repositories_file_exists = fs::exists( repositories_file )?;

    if repositories_file_exists {
        let repositories_content = fs::read_to_string( repositories_file )?;
        for repository in repositories_content.lines() {
            repositories.insert( repository.to_owned() );
        }
    }

    Ok( repositories )
}
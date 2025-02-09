use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fs;
use std::process::Command;

use clap::Parser;
use clap::Subcommand;

use git2::BranchType;
use git2::Repository;
use git2::RepositoryState;

use repositories::discover;
use repositories::load_known_repositories;

mod repositories;

#[ derive( Parser ) ]
#[ command( author, version, about, long_about = None ) ]
struct Cli {
    /// Print where the program is currently scanning.
    #[ arg( short, long ) ]
    verbose: bool,
    #[ command( subcommand ) ]
    mode: Commands,
}

#[ derive( Subcommand ) ]
enum Commands {
    /// Traverse directories, list detected changes (`-h` for modifiers)
    Discover {
        /// Add all newly detected repositories.
        #[ arg( short, long ) ]
        add: bool,
        /// Remove all previously known repositories that don't exist anymore.
        #[ arg( short, long ) ]
        prune: bool,
    },
    /// Print the status of each repository.
    Status,
    /// Fetch each repository.
    Fetch,
    /// List all known repositories.
    List{
        /// Don't restrict to repositories of the current working directory.
        #[ arg( short, long ) ]
        global: bool
    },
}

fn main() -> Result<(),Box<dyn Error>> {

    let cli = Cli::parse();

    let working_directory = env::current_dir()?;

    let Some( working_directory_string ) = working_directory.to_str() else {
        panic!( "Failed to obtain string representation of the working directory!" )
    };

    // TODO: Test whether we are in a subdirectory of a git repository. This
    // should be reported to the user as an error.

    let Some( home_directory ) = home::home_dir() else {
        panic!( "Failed to obtain the user's home directory!" )
    };

    let repositories_file = home_directory.join( ".reposcanconfig" );

    let mut all_known_repositories = load_known_repositories( &repositories_file )?;

    let repositories_in_working_directory: BTreeSet<String> =
        all_known_repositories.iter()
            .filter_map(
                | repository |
                if repository.starts_with( working_directory_string ) {
                    Some( repository.clone() )
                } else {
                    None
                }
            ).collect();

    let ignored_repositories_count =
        all_known_repositories.len() - repositories_in_working_directory.len();

    match &cli.mode {
        Commands::Discover {
            add,
            prune,
        } => {
            
            let discovered_repositories: Vec<String> =
                discover( &working_directory, cli.verbose )?.into_iter().map(
                    | repository | {
                        repository.to_str().unwrap().to_owned()
                    }
                ).collect();

            let new_repositories: Vec<String> = discovered_repositories.iter()
                .filter_map(
                    | repository |
                    if !repositories_in_working_directory.contains( repository ) {
                        Some( repository.clone() )
                    } else {
                        None
                    }
                ).collect();
            let obsolete_repositories: Vec<String> = repositories_in_working_directory.iter()
                .filter_map(
                    | repository |
                    if !discovered_repositories.contains( repository ) {
                        Some( repository.clone() )
                    } else {
                        None
                    }
                )
                .collect();

            if *add {
                for new_repository in &new_repositories {
                    if !all_known_repositories.contains( new_repository ) {
                        all_known_repositories.insert( new_repository.clone() );
                        println!( "Added new repository: \"{}\"", new_repository );
                    }
                }
            }
            println!();
            if *prune {
                for obsolete_repository in &obsolete_repositories {
                    all_known_repositories.remove( obsolete_repository );
                    println!( "Removed obsolete repository: \"{}\"", obsolete_repository );
                }
            }

            if !add && !prune {

                println!( "NEW repositories:" );
                new_repositories.iter().for_each(
                    | new_repository |
                    println!( "{new_repository}" )
                );
                println!();

                println!( "OBSOLETE repositories:" );
                obsolete_repositories.iter().for_each(
                    | obsolete_repository |
                    println!( "{obsolete_repository}" )
                );
                println!();
            }
        },
        Commands::Fetch => {

            for repository_path in &all_known_repositories {

                let repository = Repository::open( repository_path )?;
                println!(
                    "Fetching \"{}\" ... ",
                    repository_path
                );
                let branches: Vec<String> =
                    repository.branches( Some( BranchType::Local ) )?.into_iter().filter_map(
                        | branch |
                        match branch {
                            Ok( branch ) => Some( branch.0.name().unwrap().unwrap().to_owned() ),
                            Err( _ ) => None,
                        }
                    ).collect();
                let remotes: Vec<String> =
                    repository.remotes()?.into_iter().filter_map(
                        | remote |
                        match remote {
                            Some( remote ) => Some( remote.to_owned() ),
                            None => None,
                        }
                    ).collect();
                
                for remote_str in &remotes {
                    if remotes.len() > 1 {
                        println!( "(from remote {remote_str})" );
                    }
                    let mut remote = repository.find_remote( remote_str )?;

                    let fetch_result =
                        remote.fetch( &branches, None, None );
                    if let Err( _ ) = fetch_result {
                        println!( "(Trying authented fetch via a git subprocess ...) ");
                        let output_result =
                            Command::new( "git" )
                                .args( [ "fetch", remote_str ] )
                                .current_dir( repository_path )
                                .output();
                        let Ok( output ) = output_result else {
                            println!( "Failed to call on git!" );
                            continue;
                        };
                        if !output.status.success() {
                            println!( "Failed!" );
                            println!( "{:?}", output );
                        } else {
                            println!(
                                "Succeeded."
                            );
                        }
                    } else {
                        println!( "Succeeded.");
                    }
                }
                println!();
            }
        },
        Commands::Status => {
            for repository_path in &all_known_repositories {
                
                if cli.verbose {
                    println!( "Obtaining status of repository: {repository_path} ..." );
                }

                let repository = Repository::open( repository_path )?;
                let state_clean = match repository.state() {
                    RepositoryState::Clean => true,
                    _ => false,
                };
                let status_clean = repository.statuses( None )?.iter()
                    .filter(
                        | status |
                        !status.status().is_ignored()
                    )
                    .count();

                if !state_clean || status_clean != 0 {
                    println!(
                        "{} file(s) unclean @ {}",
                        status_clean,
                        repository_path,
                    );
                }
            }
        },
        Commands::List { global } => {
            let repositories_to_display =
                if *global {
                    &all_known_repositories
                } else {
                    &repositories_in_working_directory
                };
            for repository in repositories_to_display {
                println!( "{repository}" );
            }
        }
    }

    if let Commands::Discover { add, prune } = &cli.mode {
        if *add || *prune {
            let mut repositories_content = String::new();
            for repository in all_known_repositories {
                repositories_content.push_str( &repository );
                repositories_content.push( '\n' );
            }
            fs::write( &repositories_file, repositories_content )?;
        }
    }

    // Show number of ignored repositories if the `--global` switch was not
    // used.
    if let Commands::List { global: false } = &cli.mode {

        // Don't show number of ignored repositories if none were ignored.
        if ignored_repositories_count == 0 {
            return Ok( () );
        }

        println!();
        println!(
            "(Ignored {ignored_repositories_count} repositor{} which {} outside of the current working directory.)",
            if ignored_repositories_count == 1 { "y" } else { "ies" },
            if ignored_repositories_count == 1 { "is" } else { "are" },
        );
        println!();
    }

    Ok( () )
}

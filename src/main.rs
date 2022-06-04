
// FS crates
use std::fs;
use std::str;
use std::os::unix;
use fs_extra::dir;
use fs_extra::file;

// extra crates
use clap::Parser;                       // CLI parser. easy to work with.
use std::collections::HashMap;          // for hashmap support/ dictionaries. used for saving alias -> (local,remote)
use serde::{Deserialize, Serialize};    // used for serializing json. for the configuration data type
extern crate fs_extra;                  // extra functions used for symlinks.

// global variables. only for debugging and compiler-time configureations. 
static G_LOCAL_PATH: &'static str = "local";    //  
static G_CONFIG_PATH: &'static str = "/etc/";   // where the .json is stored.
static G_DEFAULT_LOCAL: &'static str = "/var/sourcerer";   // where the program moves remote files to / links point here


// clap configureation. Used for command line interface.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {

    /// unique name, remote file/dir
    #[clap(short, long, multiple_values = true, required = false)]
    add: Option<Vec<String>>,

    /// soft delete, remove link copy files back over
    #[clap(short, long, required = false)]
    remove: Option<String>,

    // the list command would list the current added aliases. 
    #[clap(short, long, required = false, parse(from_occurrences))]
    list: usize,


    #[clap(required = false)] // this is to set the local location somewhere else. needs to move to the new location
    local_location: Option<String>
}


// config is able to be expanded to 
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    //dict of aliases -> paths
    paths: HashMap<String, (String, String)>,

    local_loation : String,

    // skip_sudo : bool, // if it shoud just ignore sudo when running.


}

fn main() {

    let args = Args::parse();

    // load or new config.
    let res = load_config();
    let mut config;
    if res.is_ok() {
        config = res.unwrap();
    } else {
        config = Config {
            paths: HashMap::new(),
            local_loation: G_DEFAULT_LOCAL.to_owned()
        };
    }

    // old config loaded or new config generated


    // clap cli integration need to overhaul this garbage. not very readable but functional
    if args.add.is_some() {
        if !matches!(&args.add.clone().unwrap().len(), 2) {
            println!("Need 2 parameters: alias, path");
            return;
        } else {
            println!("adding alias...");
            add_alias(
                &mut config,
                &args.add.clone().unwrap()[0].as_str(),
                &args.add.clone().unwrap()[1].as_str(),
            )
        }
    
    } // remove config.
    else if args.remove.is_some() {
        println!("Deleting...");
        // using unwrap should be fine here as it shouldent ever error.
        soft_delete(&args.remove.unwrap(), &mut config);
    }


    // regenerate missing links.
    generate_links(&config);

    // save configureation after each run.
    let seril = save_config(&config);
    // bad way of doing a print/ list command
    if args.list != 0 {
        println!("{}", seril)
    }
}


// give it Relatve path and return result wrapped Absolute path. 
fn clean_path(path: &str) -> Result<String, std::io::Error> {
    Ok(fs::canonicalize(path)?.to_str().unwrap().to_owned())
}


// dynamic delete, delete file / folder
fn dyn_delete(path: &str) {
    // delete remote file.
    let attr = fs::metadata(path).expect("unable to fetch metadata, invalad path.");

    //check for permissions. if permissions is invalid, return or error?
    if !permission_check(path) {
        return;
    }

    // delete file/dir
    if attr.file_type().is_dir() {
        fs::remove_dir_all(path).expect("invalid path or not a folder");
    } else {
        fs::remove_file(path).expect("invalid path or not a file");
    }
}


//dynamic move, move file / dir
fn dyn_move(from: &str, to: &str) {
    let attr = fs::metadata(&from);
    if attr.is_err() {
        println!("invalid path: {}", &from);
        return;
    }
    
    // if dir
    let attr = attr.unwrap();  // !! If fail error 
    if attr.is_dir() {
        let options = dir::CopyOptions::new();
        dir::move_dir(&from, &to, &options).unwrap(); // !!
    } else {
        let options = file::CopyOptions::new();
        let to: String = to.to_owned() + "/" + from.trim_end_matches('/').split('/').last().unwrap(); // !!
        file::move_file(&from, &to, &options).unwrap();// !!
    }
}

// converts a remote link + an alias to a local link. aliases must be unique, preferably descriptive.
fn remote_to_local(path: &str, alias: &str) -> String {
    let v: Vec<&str> = path.trim_end_matches('/').split('/').collect();
    return [G_LOCAL_PATH, alias, v.last().unwrap()].join("/"); // !!
}

// permission_check function - in alias and paths, get metadata check writability.
fn permission_check(path: &str) -> bool {
    let attr = fs::metadata(path).expect("unable to fetch metadata, invalad path.");
    if attr.permissions().readonly() && !matches!(sudo::check(), sudo::RunningAs::Root) {
        // if file is root owned and user is not root
        println!("escalate");
        sudo::escalate_if_needed().expect("could not escalate");
    }
    return true;
}

// add new alias and copy to local and remove remote
fn add_alias(config: &mut Config, alias: &str, remote_path: &str) {
    // check for dupelicates err if true.
    if config.paths.contains_key(alias) || config.paths.values().any(|x| x.1.eq(remote_path)) {
        println!("duplicate, exiting...");
        return;
    }

    let remote_path = clean_path(remote_path).unwrap(); // !!

    //alias: &str,
    let attr = fs::metadata(&remote_path);
    if attr.is_err() {
        println!("invalid path");
        return;
    }
    let local_path = remote_to_local(&remote_path, alias);
    //let file_type = attr.file_type();
    if attr.unwrap().is_symlink() { // !!
        println!("path cannot be symlink. insted use file link is pointing to.");
        return;
    }

    // possibly push to another thread?
    // copy folder/file to local path under alias.

    fs::create_dir_all(&local_path.split_at(local_path.rfind('/').unwrap()).0).unwrap(); // !!

    dyn_move(&remote_path, &local_path.as_str().split_at(local_path.as_str().rfind('/').unwrap()).0); // !!
    
    //fs::copy(&remote_path, &local_path).expect("unable to copy file, invalad paths?");

    // add alias name and path to path struct. everything above was sucsessful.. hopefully
    config.paths.insert(
        alias.to_string(),
        (local_path.to_string(), remote_path.to_string()),
    );
    // migth need to clone due to borrowed string
}

fn generate_links(config: &Config) {
    for (_k, v) in config.paths.iter() {
        // k - alias, v - (local, remote)
        //first check remote to see if symlink already exists. delete if file/folder
        let attr = fs::metadata(v.1.clone());
        if attr.is_ok() {
            // remote file exists
            if !attr.unwrap().file_type().is_symlink() {
                // if remote is not symlink
                if clean_path(v.0.as_str()).is_ok() {
                    // if local version exists
                    dyn_delete(v.1.as_str()); // delete remote
                    if unix::fs::symlink(
                        &clean_path(v.0.as_str()).unwrap(),
                        v.1.as_str(),
                    ).is_err()
                    {
                        println!("Error, Could not create SymLink");
                        return;
                    }
                }
            }
            break;
        } else {
            // create remote directory if non-existant. should do permission testing? \
            if fs::create_dir_all(&v.1.split_at(v.1.rfind('/').unwrap()).0).is_err()
            {
                println!("could not create directory tree")
            }
            if unix::fs::symlink(
                &clean_path(v.0.as_str()).unwrap(), // !!
                v.1.as_str(),
            ).is_err()
            {
                println!("Error, Could not create SymLink");
                return;
            }

        }
    }
}

// func to 'soft' delete alias.
fn soft_delete(alias: &str, config: &mut Config) -> bool {
    // validity check
    if !config.paths.contains_key(alias) {
        println!("Invalid, exiting...");
        return false;
    }

    let local_path  = config.paths.get(alias).unwrap().0.clone(); // !!
    let remote_path = config.paths.get(alias).unwrap().1.clone(); // !!

    // delete remote link
    dyn_delete(&remote_path.as_str());

    // move local to remote.
    dyn_move(&clean_path(&local_path).unwrap(),&remote_path.split_at(remote_path.rfind('/').unwrap()).0); // !!

    config.paths.remove(alias);

    return true;
}


// serialize config and links save to file. 
fn save_config(config: &Config) -> String {
    let serialized = serde_yaml::to_string(&config).unwrap();
    fs::write( [G_CONFIG_PATH, "config.yaml"].join(""), &serialized).unwrap();
    serialized // return serialized string 
}


// load config. should have a default save/load dir. ie: ~/.source
fn load_config() -> Result<Config, std::io::Error> {
    let input = fs::read_to_string("config.yaml")?;
    Ok(serde_yaml::from_str(&input).unwrap())
}

use notify::{event::{DataChange, ModifyKind}, Event, EventKind, RecursiveMode, Watcher};
use std::{fs::File, io::{BufReader, Read}, path::Path, sync::mpsc};
use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;
mod parse;
mod generate;
mod types;

/// frosted: freezed light
/// watches your files and generates code only where and when needed 
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// directory in which we will watch for changes
    #[arg(short, long, value_name = "DIR")]
    directory: PathBuf,

    /// magic token to identify the files to parse
    #[arg(short, long, value_name = "TOKEN", default_value = "+mk:")]
    magic_token: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let directory = args.directory;
    let magic_token = args.magic_token;

    let file_watcher = FileWatcher::new(&directory, &magic_token)?;

    // this will run forever
    file_watcher.run()
}


struct FileWatcher {
    rx: mpsc::Receiver<notify::Result<Event>>,
    watcher: notify::RecommendedWatcher,
    magic_token: String,
}

impl FileWatcher {
    fn new(directory: &Path, magic_token: &str) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(Path::new(&directory), RecursiveMode::Recursive)?;
        // keep the watcher alive
        Ok(Self { rx, watcher, magic_token: magic_token.to_string() })
    }

    fn run(&self) -> Result<()> {
        for res in &self.rx {
            match res {
                Ok(event) => self.handle_event(&event)?,
                Err(e) => println!("watch error: {:?}", e),
            }
        }
        Ok(())
    }

    fn handle_event(&self, event: &Event) -> Result<()> {
        if let EventKind::Modify(ModifyKind::Data(DataChange::Content)) = event.kind {
            self.handle_file_change(event.paths.first().ok_or(anyhow::anyhow!("No path found"))?)?;
        }
        Ok(())
    }

    fn handle_file_change(&self, path: &Path) -> Result<()> {
        self.parse_file(path)?;
        Ok(())
    }

    fn parse_file(&self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut code = String::new();
        reader.read_to_string(&mut code)?;
    
        println!("parsing file: {:?}", path);
        let classes = parse::parse(&code, &self.magic_token)?;
        println!("classes: {:?}", classes);

        
        Ok(())
    }
}
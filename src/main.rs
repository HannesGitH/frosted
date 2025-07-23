use notify::{event::{DataChange, ModifyKind}, Event, EventKind, RecursiveMode, Watcher};
use std::{fs::File, io::{BufReader, Read, Write}, path::Path, sync::mpsc};
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

    /// file extension to name the generated file
    #[arg(short, long, value_name = "OUTPUT", default_value = "copy.gen.dart")]
    output_file_extension: String,

    /// files extensions to watch for changes
    #[arg(short, long, value_name = "EXTENSIONS", default_value = "dart")]
    file_extensions: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let directory = args.directory;
    let magic_token = args.magic_token;
    let output_file_extension = args.output_file_extension;
    let file_extensions = args.file_extensions;

    let file_watcher = FileWatcher::new(&directory, &magic_token, &output_file_extension, &file_extensions)?;

    // this will run forever
    file_watcher.run()
}


struct FileWatcher {
    rx: mpsc::Receiver<notify::Result<Event>>,
    // just here to keep the watcher alive
    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,
    magic_token: String,
    output_file_extension: String,
    allowed_watch_extensions: Vec<String>,
}

impl FileWatcher {
    fn new(directory: &Path, magic_token: &str, output_file_extension: &str, file_extensions: &[String]) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(Path::new(&directory), RecursiveMode::Recursive)?;
        // keep the watcher alive
        Ok(Self { rx, watcher, magic_token: magic_token.to_string(), output_file_extension: output_file_extension.to_string(), allowed_watch_extensions: file_extensions.to_vec() })
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
        if path.to_str().unwrap().ends_with(&self.output_file_extension) {
            return Ok(());
        }

        if !self.allowed_watch_extensions.iter().any(|ext| path.extension().unwrap_or_default().to_str().unwrap().ends_with(ext)) {
            return Ok(());
        }

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

        if classes.is_empty() {
            return Ok(());
        }

        let generated = generate::generate(&classes, path.file_name().unwrap().to_str().unwrap())?;
        let output_path = path.with_extension(&self.output_file_extension);
        let mut file = File::create(output_path)?;
        file.write_all(generated.as_bytes())?;
        
        Ok(())
    }
}
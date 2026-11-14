use architect_design::catalog::{self, Selection};
use architect_design::export::Format;
use architect_design::locale::Config;
use architect_design::{assets, audit, error::Result, util};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "Offline SVG renderer and validators for the architecture design system")]
struct Cli {
    /// Repository root (default: ARCHITECT_DESIGN_ROOT or the nearest ancestor holding assets/design-tokens.json)
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Args)]
struct Locales {
    /// Output locale (en, ja-JP; `ja_jp` also accepted). Default: per-case setting, else config/diagram.json
    #[arg(long, conflicts_with = "all_locales")]
    locale: Option<String>,
    /// Render every configured locale
    #[arg(long)]
    all_locales: bool,
}

impl Locales {
    fn selection(&self) -> Selection<'_> {
        if self.all_locales {
            Selection::All
        } else {
            Selection::One(self.locale.as_deref())
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Render a v2 model to SVG (Japanese by default: diagram.ja_jp.svg)
    Render {
        model: PathBuf,
        /// Output path (default: diagram[.<suffix>].svg next to the model; one locale only)
        #[arg(long, conflicts_with = "all_locales")]
        output: Option<PathBuf>,
        /// Formats: svg, mermaid, drawio (comma separated)
        #[arg(long, value_delimiter = ',', default_value = "svg")]
        format: Vec<String>,
        #[command(flatten)]
        locales: Locales,
    },
    /// Check v2 model structure and model/SVG parity for every maintained diagram
    ValidateDiagrams,
    /// Check the icon manifest, files, SHA-256 and example references
    ValidateAssets,
    /// Rebuild the component catalogue and re-render every example SVG (Japanese by default)
    Examples {
        #[command(flatten)]
        locales: Locales,
    },
}

fn run(cli: Cli) -> Result<()> {
    let root = cli.root.unwrap_or_else(util::find_root);
    let tokens = util::tokens(&root)?;
    match cli.command {
        Command::Render { model, output, format, locales } => {
            let config = Config::load(&root)?;
            let formats = format.iter().map(|f| Format::parse(f)).collect::<Result<Vec<_>>>()?;
            if output.is_some() && formats.len() > 1 {
                return architect_design::error::err("--output takes a single format");
            }
            for def in catalog::locales_for(&config, &model, &locales.selection())? {
                for format in &formats {
                    let target = catalog::render_model(&root, &tokens, &config, &model, def, *format, output.as_deref())?;
                    println!("{}", target.display());
                }
            }
            Ok(())
        }
        Command::ValidateDiagrams => {
            let count = audit::validate_diagrams(&root, &tokens)?;
            println!("PASS: {count} v2 diagrams; locales, regions, ports, rightward flow, SVG parity across languages, estimated text bounds and intersections");
            Ok(())
        }
        Command::ValidateAssets => {
            let count = assets::validate_assets(&root)?;
            println!("PASS: {count} icons; manifest/files/SHA-256/SVG/PNG/collections/example/architecture references and SVG/draw.io embeddings");
            Ok(())
        }
        Command::Examples { locales } => {
            for name in catalog::render_examples(&root, &tokens, &locales.selection())? {
                println!("{name}");
            }
            Ok(())
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

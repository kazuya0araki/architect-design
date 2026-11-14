use architect_design::{assets, audit, catalog, error::Result, render, util, xml};
use clap::{Parser, Subcommand};
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

#[derive(Subcommand)]
enum Command {
    /// Render a v2 model to SVG
    Render {
        model: PathBuf,
        /// Output path (default: diagram.svg next to the model)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Check v2 model structure and model/SVG parity for every maintained diagram
    ValidateDiagrams,
    /// Check the icon manifest, files, SHA-256 and example references
    ValidateAssets,
    /// Rebuild the component catalogue and re-render every example SVG
    Examples,
}

fn run(cli: Cli) -> Result<()> {
    let root = cli.root.unwrap_or_else(util::find_root);
    let tokens = util::tokens(&root)?;
    match cli.command {
        Command::Render { model, output } => {
            let svg = render::render(&util::read_json(&model)?, &root, &tokens)?;
            xml::write_file(&svg, &output.unwrap_or_else(|| model.with_file_name("diagram.svg")))
        }
        Command::ValidateDiagrams => {
            let count = audit::validate_diagrams(&root, &tokens)?;
            println!("PASS: {count} v2 diagrams; regions, ports, rightward flow, SVG parity, estimated text bounds and intersections");
            Ok(())
        }
        Command::ValidateAssets => {
            let count = assets::validate_assets(&root)?;
            println!("PASS: {count} icons; manifest/files/SHA-256/SVG/PNG/collections/example/architecture references and SVG/draw.io embeddings");
            Ok(())
        }
        Command::Examples => {
            for name in catalog::render_examples(&root, &tokens)? {
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

use bloomfilter::Bloom;
use clap::{ArgGroup, Args, Parser, Subcommand};
use colored::*;
use dtl_hunter::{
    check_val_in_bloom, get_bloom_from_path, lookup_values_in_dtl, read_input_file,
    write_bloom_to_file, write_csv,
};
use log::{error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
#[derive(Parser)]
#[clap(
    name = "Datalake Hunter",
    author = "orangecyberdefense.com",
    version = "1.0",
    about = "Allow to mass check data from datalake using bloom filters.",
    long_about = None
)]

// #[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    #[clap(
        short,
        long,
        value_parser(["prod", "preprod"]),
        help = "Datalake API environment.",
        global = true,
        default_value = "prod"
    )]
    environment: String,
}

#[derive(Subcommand)]
enum Commands {
    Check(Check),
    Create(Create),
    Lookup(Lookup),
}

#[derive(Args)]
#[clap(
    about = "Checks if values in the provided file can be found in bloom filters or in Datalake using query hashes."
)]
#[clap(group(ArgGroup::new("bloom_filter_group").required(true).args(&["bloom", "queryhash"])))]
struct Check {
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to file to which the list of matched inputs will be pushed to as a csv file."
    )]
    output: Option<PathBuf>,
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to file containing the value to check, one value per line or the values from the first column in a CSV."
    )]
    input: PathBuf,
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to a bloom filter to be used for the check. Required if no query hashes are provided"
    )]
    bloom: Option<Vec<std::path::PathBuf>>,
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Query hash from which to build a bloom filter. Required if no bloom filter files are provided."
    )]
    queryhash: Option<String>,
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to the file in which Lookup matched values should be written."
    )]
    lookup: Option<std::path::PathBuf>,
    #[clap(long, help = "Silence the output of matched value to the stdout.")]
    quiet: bool,
    #[clap(long = "no-header", help = "Remove the header in the output csv file.")]
    no_header: bool,
}

#[derive(Args)]
#[clap(about = "Creates a bloom filter from a provided query hash or file.")]
#[clap(group(ArgGroup::new("create_use_either").required(true).args(&["file", "queryhash"])))]
struct Create {
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Query hash from which to build a bloom filter."
    )]
    queryhash: Option<String>,
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to the file to output the created bloom filter."
    )]
    output: PathBuf,
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to file containing the value to check, one value per line or the values from the first column in a CSV."
    )]
    file: Option<std::path::PathBuf>,
    #[clap(
        short,
        long,
        value_parser =  validate_false_positive,
        forbid_empty_values = true,
        default_value = "0.00001",
        help = "Rate of false positive. Can be between 0.0 and 1.0. The lower the rate the bigger the bloom filter will be."
    )]
    rate: f64,
}

#[derive(Args)]
#[clap(about = "Makes a lookup in Datalake on provided values.")]
struct Lookup {
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to file containing the value to check, one value per line or the values from the first column in a CSV."
    )]
    input: PathBuf,
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to a CSV file in which to output the result."
    )]
    output: PathBuf,
}

fn validate_false_positive(value: &str) -> Result<f64, String> {
    let fp: f64 = value.parse().map_err(|_| {
        format!(
            "False positive rate should be between 0.0 an 1.0, {} was provided",
            value
        )
    })?;
    if fp > 0.0 && fp < 1.0 {
        Ok(fp)
    } else {
        Err(format!(
            "False positive rate should be between 0.0 an 1.0, {} was provided",
            value
        ))
    }
}

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let cli: Cli = Cli::parse();
    match &cli.command {
        Commands::Check(args) => check_command(args, &cli),
        Commands::Create(args) => create_command(args, &cli),
        Commands::Lookup(args) => lookup_command(args, &cli),
    }
}

fn create_command(args: &Create, cli: &Cli) {
    let bloom_result = if let Some(queryhash) = &args.queryhash {
        dtl_hunter::create_bloom_from_queryhash(queryhash.clone(), &cli.environment, args.rate)
    } else if let Some(input_path) = &args.file {
        dtl_hunter::create_bloom_from_file(input_path, args.rate)
    } else {
        error!("Unexpected case");
        return;
    };
    match bloom_result {
        Ok(bloom) => write_bloom(bloom, &args.output),
        Err(e) => {
            error!("Error while creating bloom filter: {}", e)
        }
    };
}

fn write_bloom(bloom: Bloom<String>, output: &PathBuf) {
    match write_bloom_to_file(bloom, output) {
        Ok(()) => {
            println!(
                "{}{}",
                "Successfully create the bloomfilter at path: "
                    .green()
                    .bold(),
                &output.display()
            );
        }
        Err(e) => error!("{}", e),
    }
}

fn check_command(args: &Check, _cli: &Cli) {
    let input: Vec<String> = match read_input_file(&args.input) {
        Ok(input) => input,
        Err(e) => {
            error!("{}: {}", &args.input.display(), e);
            return;
        }
    };

    let mut blooms: HashMap<String, Bloom<String>> = HashMap::new();

    if let Some(bloom_paths) = &args.bloom {
        let file_blooms = match get_bloom_from_path(bloom_paths) {
            Ok(file_bloom) => file_bloom,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
        blooms.extend(file_blooms);
    }
    if args.queryhash.is_some() {}

    let mut bloom_matches: HashMap<String, Vec<String>> = HashMap::new();
    let mut nb_matches: usize = 0;

    for (filename, bloom) in blooms {
        let matches: Vec<String> = check_val_in_bloom(bloom, &input);
        nb_matches += matches.len();
        bloom_matches.insert(filename, matches);
    }
    manage_check_output(
        &args.output,
        bloom_matches,
        args.quiet,
        args.no_header,
        nb_matches,
    )
}

fn manage_check_output(
    output_path: &Option<PathBuf>,
    bloom_matches: HashMap<String, Vec<String>>,
    quiet: bool,
    no_header: bool,
    nb_matches: usize,
) {
    info!(
        "{} - {}",
        format!("{} matches", &nb_matches).bright_blue().bold(),
        "Be advised that some matches might be false positives."
            .yellow()
            .italic()
            .dimmed()
    );
    if let Some(output) = output_path {
        if nb_matches > 0 {
            match write_csv(&bloom_matches, output, &no_header) {
                Ok(()) => {
                    info!(
                        "{} {}",
                        "Results saved in".green().bold(),
                        &output.display()
                    )
                }
                Err(e) => error!("{}", e),
            }
        } else {
            warn!("{}", "No matches, output file was not created".yellow());
        }
    }
    if !quiet {
        for (filename, values) in bloom_matches {
            for val in values {
                println!("{},{}", val, filename);
            }
        }
    }
}

fn lookup_command(args: &Lookup, cli: &Cli) {
    let input: Vec<String> = match read_input_file(&args.input) {
        Ok(input) => input,
        Err(e) => {
            error!("{}: {}", &args.input.display(), e);
            return;
        }
    };
    _ = lookup_values_in_dtl(input, &args.output, &cli.environment);
}

#[test]
fn test_validate_false_positive_rate() {
    assert!(validate_false_positive("0.0").is_err());
    assert!(validate_false_positive("1.0").is_err());
    assert!(validate_false_positive("2.5").is_err());
    assert!(validate_false_positive("0.0000001").is_ok());
}

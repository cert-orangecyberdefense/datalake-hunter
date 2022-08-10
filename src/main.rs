use clap::{ArgGroup, Args, Parser, Subcommand};
use log::error;
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
    about = "Checks if atom values in the provided file can be found in one or more provided bloom filter or in a bloom filter generated from a query hash."
)]
#[clap(group(ArgGroup::new("bloom_filter_group").required(true).args(&["bloom", "queryhash"])))]
struct Check {
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to file to which the list of matching inputs will be pushed to as a csv file."
    )]
    output: std::path::PathBuf,
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to file containing the value to check, one value per line."
    )]
    input: std::path::PathBuf,
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to a bloom filter to be used for the check. Required if no query hash are provided"
    )]
    bloom: Option<Vec<std::path::PathBuf>>,
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Query hash from which to build a bloom filter. Required if no bloom filter file are provided."
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
    output: std::path::PathBuf,
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to the file to use to created a bloom filter. One value per line."
    )]
    file: Option<std::path::PathBuf>,
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Rate of false positive. The lower the rate the bigger the bloom filter will be."
    )]
    positive: f64,
}

#[derive(Args)]
#[clap(about = "Makes a lookup in Datalake on provided values.")]
struct Lookup {
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to a CSV file containing the value to lookup in Datalake."
    )]
    input: std::path::PathBuf,
    #[clap(
        short,
        long,
        value_parser,
        forbid_empty_values = true,
        help = "Path to a CSV file in which to output the result."
    )]
    output: std::path::PathBuf,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    match &cli.command {
        Commands::Check(args) => check_command(args, &cli),
        Commands::Create(args) => create_command(args, &cli),
        Commands::Lookup(args) => {
            println!("Lookup was used {:?}", args.input)
        }
    }
}

fn check_command(args: &Check, _cli: &Cli) {
    let _input: Vec<String> = match datalake_hunter::read_input(&args.input) {
        Ok(input) => input,
        Err(e) => {
            error!("{}: {}", &args.input.display(), e);
            return;
        }
    };
    if args.bloom.is_some() {}
    if args.queryhash.is_some() {}
}

fn create_command(args: &Create, _cli: &Cli) {
    if args.queryhash.is_some() {
        println!("queryhash");
    }
    if let Some(input_path) = &args.file {}
}

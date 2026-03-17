use std::fs::File;
use std::io::{self, BufReader};

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FileFormat {
    Csv,
    Txt,
    Bin,
}

#[derive(Debug, Parser)]
#[command(name = "ypbank_converter")]
#[command(about = "Конвертер финансовых транзакций между форматами")]
struct CliArgs {
    #[arg(long)]
    input: String,

    #[arg(long = "input-format", value_enum)]
    input_format: FileFormat,

    #[arg(long = "output-format", value_enum)]
    output_format: FileFormat,
}

fn main() {
    if let Err(error_message) = run() {
        eprintln!("{error_message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli_args = CliArgs::parse();

    let input_file = File::open(&cli_args.input).map_err(|open_error| {
        format!("Ошибка открытия файла {}: {}", cli_args.input, open_error)
    })?;

    let buffered_reader = BufReader::new(input_file);

    let transactions = match cli_args.input_format {
        FileFormat::Csv => yandex_bank::read_csv(buffered_reader),
        FileFormat::Txt => yandex_bank::read_txt(buffered_reader),
        FileFormat::Bin => yandex_bank::read_bin(buffered_reader),
    }
    .map_err(|read_error| format!("Ошибка при чтении: {read_error}"))?;

    let stdout = io::stdout();
    let stdout_lock = stdout.lock();

    match cli_args.output_format {
        FileFormat::Csv => yandex_bank::write_csv(stdout_lock, &transactions),
        FileFormat::Txt => yandex_bank::write_txt(stdout_lock, &transactions),
        FileFormat::Bin => yandex_bank::write_bin(stdout_lock, &transactions),
    }
    .map_err(|write_error| format!("Ошибка при записи: {write_error}"))?;

    Ok(())
}

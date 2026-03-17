use std::fs::File;
use std::io::BufReader;

use clap::{Parser, ValueEnum};
use yandex_bank::Transaction;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FileFormat {
    Csv,
    Txt,
    Bin,
}

#[derive(Debug, Parser)]
#[command(name = "ypbank_compare")]
#[command(about = "Сравнение файлов с транзакциями в форматах YPBank")]
struct CliArgs {
    #[arg(long)]
    file1: String,

    #[arg(long = "format1", value_enum)]
    format1: FileFormat,

    #[arg(long)]
    file2: String,

    #[arg(long = "format2", value_enum)]
    format2: FileFormat,
}

fn main() {
    if let Err(error_message) = run() {
        eprintln!("{error_message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli_args = CliArgs::parse();

    let transactions1 = read_transactions(&cli_args.file1, cli_args.format1)?;
    let transactions2 = read_transactions(&cli_args.file2, cli_args.format2)?;

    compare_transactions(
        &transactions1,
        &transactions2,
        &cli_args.file1,
        &cli_args.file2,
    )?;

    println!(
        "Файлы '{}' и '{}' содержат одинаковые транзакции.",
        cli_args.file1, cli_args.file2
    );

    Ok(())
}

fn read_transactions(file_path: &str, file_format: FileFormat) -> Result<Vec<Transaction>, String> {
    let input_file = File::open(file_path)
        .map_err(|open_error| format!("Ошибка открытия файла {}: {}", file_path, open_error))?;

    let buffered_reader = BufReader::new(input_file);

    match file_format {
        FileFormat::Csv => yandex_bank::read_csv(buffered_reader),
        FileFormat::Txt => yandex_bank::read_txt(buffered_reader),
        FileFormat::Bin => yandex_bank::read_bin(buffered_reader),
    }
    .map_err(|read_error| format!("Ошибка чтения файла {}: {}", file_path, read_error))
}

fn compare_transactions(
    transactions1: &[Transaction],
    transactions2: &[Transaction],
    file1: &str,
    file2: &str,
) -> Result<(), String> {
    if transactions1.len() != transactions2.len() {
        return Err(format!(
            "Файлы '{}' и '{}' различаются по количеству транзакций: {} != {}",
            file1,
            file2,
            transactions1.len(),
            transactions2.len()
        ));
    }

    for (transaction_index, (left_transaction, right_transaction)) in
        transactions1.iter().zip(transactions2.iter()).enumerate()
    {
        if left_transaction != right_transaction {
            return Err(format!(
                "Транзакция №{} не совпадает.\nФайл 1: {:?}\nФайл 2: {:?}",
                transaction_index + 1,
                left_transaction,
                right_transaction
            ));
        }
    }

    Ok(())
}

# yandex_bank

Набор crate-ов на Rust для чтения, сериализации, десериализации и сравнения финансовых транзакций в нескольких форматах:

- `CSV`
- `TXT`
- `BIN`

Проект состоит из трех crate'ов:

- `parser` - библиотека с парсерами и сериализаторами
- `converter_cli` - консольное приложение для конвертации между форматами
- `comparer_cli` - консольное приложение для сравнения файлов с транзакциями

## Сборка проекта
`cargo build`

## Запуск тестов 
`cargo test`

## Запуск
конвертор:
```
cargo run -p converter_cli -- \
  --input <input_file> \
  --input-format <csv|txt|bin> \
  --output-format <csv|txt|bin>
```

comparer:
```
cargo run -p comparer_cli -- \
  --file1 <file_path> \
  --format1 <csv|txt|bin> \
  --file2 <file_path> \
  --format2 <csv|txt|bin>
```
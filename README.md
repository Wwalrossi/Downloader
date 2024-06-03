# Downloader

## Описание
`Downloader` — это утилита для загрузки файлов с интернета с поддержкой многоядерной загрузки, ограничения скорости и возобновления загрузки. Она предназначена для повышения эффективности загрузок и обеспечения стабильности при прерывистом соединении.

## Функции
- **Многоядерная загрузка**: Поддержка параллельного скачивания, что ускоряет процесс загрузки.
- **Ограничение скорости**: Возможность задать максимальную скорость загрузки.
- **Возобновление загрузки**: Поддержка возобновления загрузки с места остановки.
- **Отслеживание прогресса**: Вывод информации о прогрессе загрузки в консоль.


## Использование
1. Запустите утилиту:<br>
downloader.exe<br>
начнется загрузка файла в C:\download<br>

2. Пример работы утилиты
При запуске утилиты, вы увидите следующую информацию в терминале:

Подготовка к загрузке
Общий размер: 0.0 Мб
Начало загрузки
Прогресс загрузки: 10% (50/500)
Прогресс загрузки: 20% (100/500)
...
Начало ограничения скорости
Снятие ограничения скорости
Причина завершения загрузки: Success

3. Настройка
   
Вы можете изменить настройки  (/src/main.rs),  количество параллельных подключений, размер блока и ограничение скорости, в исходном коде перед компиляцией. Вот основные параметры, которые можно изменить:

Размер блока (chunk_size): Размер каждого блока данных при загрузке.<br>
Количество параллельных подключений (download_connection_count): Количество одновременных соединений для загрузки.<br>
Ограничение скорости (DownloadSpeedLimiterExtension): Установить максимальную скорость загрузки в байтах в секунду.<br>
Пример изменения настроек в коде:<br>


HttpDownloaderBuilder::new(test_url.clone(), save_dir)<br>
    .chunk_size(NonZeroUsize::new(1024 * 1024 * 10).unwrap()) // Размер блока<br>
    .download_connection_count(NonZeroU8::new(3).unwrap())    // Количество параллельных подключений
    .build((<br>
        DownloadStatusTrackerExtension { log: true },       // Отслеживание статуса загрузки<br>
        DownloadSpeedTrackerExtension { log: true },       // Отслеживание скорости загрузки<br>
        DownloadSpeedLimiterExtension::new(None),          // Ограничение скорости загрузки<br>
        DownloadBreakpointResumeExtension {
            download_archiver_builder: BsonFileArchiverBuilder::new(ArchiveFilePath::Suffix("bson".to_string()))<br>
        }<br>
    ));

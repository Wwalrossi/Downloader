use std::num::{NonZeroU8, NonZeroUsize};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::time::sleep;
use url::Url;

use http_downloader::{
    breakpoint_resume::DownloadBreakpointResumeExtension,
    HttpDownloaderBuilder,
    speed_tracker::DownloadSpeedTrackerExtension,
    status_tracker::DownloadStatusTrackerExtension,
};
use http_downloader::bson_file_archiver::{ArchiveFilePath, BsonFileArchiverBuilder};
use http_downloader::speed_limiter::DownloadSpeedLimiterExtension;

#[tokio::main]
async fn main() -> Result<()> {
    let save_dir = PathBuf::from("C:/download");
    let test_url = Url::parse("https://huggingface.co/TheBloke/phi-2-GGUF/resolve/main/phi-2.Q3_K_S.gguf")?;
    let (mut downloader, (status_state, speed_state, speed_limiter, ..)) =
        HttpDownloaderBuilder::new(test_url.clone(), save_dir)
            .chunk_size(NonZeroUsize::new(1024 * 1024 * 10).unwrap()) // block size
            .download_connection_count(NonZeroU8::new(3).unwrap())    // Количество параллельных подключений
            .build((
                DownloadStatusTrackerExtension { log: true },       // Отслеживание статуса загрузки
                DownloadSpeedTrackerExtension { log: true },       // Отслеживание скорости загрузки
                DownloadSpeedLimiterExtension::new(None),          // Ограничение скорости загрузки
                DownloadBreakpointResumeExtension {
                    download_archiver_builder: BsonFileArchiverBuilder::new(ArchiveFilePath::Suffix("bson".to_string()))
                }
            ));

    println!("Подготовка к загрузке");
    let download_future = downloader.prepare_download()?;

    let _status = status_state.status(); // Получение статуса загрузки
    let _status_receiver = status_state.status_receiver; // Получение отслеживателя статуса загрузки
    let _byte_per_second = speed_state.download_speed(); // Получение скорости загрузки, байты в секунду
    let _speed_receiver = speed_state.receiver; // Получение отслеживателя скорости загрузки

    // Создание прогресс-бара
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⣀⣤⣤⣤⣀⢀⡀⠄⠂⠁")
            .template("{spinner:.green} [{elapsed}] {msg}"),
    );
    pb.enable_steady_tick(100);

    // Печать прогресса загрузки
    tokio::spawn({
        let mut downloaded_len_receiver = downloader.downloaded_len_receiver().clone();
        let total_size_future = downloader.total_size_future();
        let pb = pb.clone();
        async move {
            let total_len = total_size_future.await;
            if let Some(total_len) = total_len {
                pb.set_message(format!("Общий размер: {:.2} Мб", total_len.get() as f64 / 1024_f64 / 1024_f64));
            }

            let mut last_print_time = Instant::now();
            let mut last_percent_done = 0; // Переменная для хранения последнего процента
            while downloaded_len_receiver.changed().await.is_ok() {
                let progress = *downloaded_len_receiver.borrow();
                if let Some(total_len) = total_len {
                    let percent_done = (progress as f64 / total_len.get() as f64 * 100.0).round() as i64; // Округление до целого числа
                    if percent_done > last_percent_done {
                        pb.set_message(format!("Прогресс загрузки: {}% ({}/{})", percent_done, progress, total_len.get()));
                        last_percent_done = percent_done; // Обновление последнего процента
                        last_print_time = Instant::now();
                    }
                }

                sleep(Duration::from_millis(100)).await;
            }
            pb.finish_with_message("Загрузка завершена");
        }
    });

    // Ограничение скорости загрузки
    tokio::spawn(async move {
        sleep(Duration::from_secs(2)).await;
        println!("Начало ограничения скорости");
        speed_limiter.change_speed(Some(1024 * 1024 * 2)).await;
        sleep(Duration::from_secs(4)).await;
        println!("Снятие ограничения скорости");
        speed_limiter.change_speed(None).await;
    });

    println!("Начало загрузки");
    let dec = download_future.await?;
    println!("Причина завершения загрузки: {:?}", dec);
    Ok(())
}

use teloxide::net::Download;
use teloxide::types::ReplyParameters;
use teloxide::{
    dptree::endpoint,
    types::{InputFile, ParseMode, Seconds, Video},
    utils::command::BotCommands,
};
use teloxide::{prelude::*, DownloadError};

use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use tokio::fs;
use tokio::fs::File;

use log::{error, info, warn};

mod square_video;
use square_video::make_square_video;

const MAX_VIDEO_SIZE_BYTES: u32 = 50 * 1024 * 1024;
const VIDEO_NOTE_SIDELENGHT: u32 = 384;

const DONATION_MESSAGE: &str = concat!(
    "Поддержать разработчика бота с помощью *Monero*:```\n",
    "8BnPQDMFf8yBUg33ZuEVyFHmgYUufcRyaUpRXDXvmsca3HcKHt6tM3oBJULmDnwQELFx3mDkkjcyMezhnNcpqvMsU86zRHJ",
    "\n```"
);

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Start bot")]
    Start,
    #[command(description = "Get help")]
    Help,
}

async fn send_donation_message(bot: Bot, chat_id: ChatId) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, DONATION_MESSAGE)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

async fn start_help_handler(bot: Bot, message: Message) -> Result<(), teloxide::RequestError> {
    let first_name = match message.from {
        Some(user) => user.clone().first_name,
        None => "Unknown".to_string(),
    };

    bot.send_message(
        message.chat.id,
        format!("Привет\\! *{}*\nЭтот бот умеет ковертировать твоё видео в кружок _\\(Видеосообщение\\)_\n*Просто отправь мне Медиафайл\\.*\n", first_name.replace('_', "\\_")),
    )
    .reply_parameters(ReplyParameters::new(message.id))
    .parse_mode(ParseMode::MarkdownV2)
    .await?;

    send_donation_message(bot, message.chat.id).await?;

    Ok(())
}

async fn download_video_as_file(bot: &Bot, video: &Video) -> Result<PathBuf, DownloadError> {
    let file_id = video.file.id.clone();

    // This will now convert reqwest::Error to DownloadError automatically
    let file_object = match bot.get_file(file_id).await {
        Ok(file_object) => file_object,
        Err(err) => {
            error!("error when getting file: {}", err);
            let io_err = Error::from(ErrorKind::Other);
            return Err(DownloadError::from(io_err));
        }
    };

    let download_path = PathBuf::from("temp").join(format!("{}.mp4", file_object.id));
    info!("download path: {}", download_path.display());

    let mut file = File::create(&download_path).await.map_err(|e| {
        error!("Failed to create file for writing: {:?}", e);
        DownloadError::from(e) // Convert IoError to DownloadError
    })?;

    // Handle the download_file result
    bot.download_file(&file_object.path, &mut file)
        .await
        .map_err(|err| {
            error!("Failed to download file: {:?}", err);
            // Create an std::io::Error from the ErrorKind
            let io_err = Error::from(ErrorKind::Other);
            // Convert the std::io::Error into a DownloadError
            DownloadError::from(io_err)
        })?;

    Ok(download_path)
}

async fn video_preccessing_error(bot: Bot, message: Message) -> Result<(), teloxide::RequestError> {
    bot.send_message(message.chat.id, "*Ошибка при обработке видео\\!*")
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

async fn handle_square_video_download(
    bot: Bot,
    message: Message,
    download_result: Result<PathBuf, DownloadError>,
) -> Result<(), teloxide::RequestError> {
    match download_result {
        Ok(download_path) => {
            info!(
                "File downloaded successfully to {}",
                download_path.display()
            );
            let input_file = InputFile::file(&download_path);
            bot.send_video_note(message.chat.id, input_file).await?;
            info!("Video note sent successfully sent.");
            match fs::remove_file(&download_path).await {
                Ok(_) => info!("File successfully removed"),
                Err(err) => error!("Failed to remove file: {}", err),
            };
            send_donation_message(bot, message.chat.id).await?;
        }
        Err(err) => {
            error!("Failed to download file: {:?}", err);
            video_preccessing_error(bot, message).await?;
        }
    }
    Ok(())
}

async fn handle_non_square_video_download(
    bot: Bot,
    message: Message,
    download_result: Result<PathBuf, DownloadError>,
) -> Result<(), teloxide::RequestError> {
    match download_result {
        Ok(download_path) => {
            info!(
                "File downloaded successfully to {}",
                download_path.display()
            );
            match make_square_video(&download_path, VIDEO_NOTE_SIDELENGHT).await {
                Ok(square_video_path) => {
                    info!("Square video path: {}", square_video_path.display());

                    // Now upload the downloaded file as a video note
                    let input_file = InputFile::file(&square_video_path);
                    bot.send_video_note(message.chat.id, input_file).await?;
                    info!("Video note sent successfully.");

                    send_donation_message(bot, message.chat.id).await?;

                    match fs::remove_file(&square_video_path).await {
                        Ok(_) => info!("File successfully removed"),
                        Err(err) => error!("{}", err),
                    };
                }
                Err(err) => {
                    error!("Failed to create square video: {:?}", err);
                    video_preccessing_error(bot, message).await?;
                }
            }

            return Ok(());
        }
        Err(err) => {
            error!("Error occurred when downloading file: {}", err);
            video_preccessing_error(bot, message).await?;
        }
    }
    Ok(())
}

async fn video_handler(bot: Bot, message: Message) -> Result<(), teloxide::RequestError> {
    if let Some(video) = message.clone().video() {
        // Check if the video meets the criteria for a video note
        if video.file.size > MAX_VIDEO_SIZE_BYTES {
            info!("Video size exceeds 50 MB, not processing.");
            bot.send_message(message.chat.id, "*Размер видео превышает 50 МБ!*")
                .reply_parameters(ReplyParameters::new(message.id))
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        }

        // if telegram api video_note duration limit is satisfied
        if video.duration <= Seconds::from_seconds(60) {
            // if width and height are equal, we can send video as video_note directly
            if video.width == video.height {
                bot.send_message(message.chat.id, "*Подождите\\.\\.\\.*")
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?;

                let download_result = download_video_as_file(&bot, video).await;

                return handle_square_video_download(bot, message, download_result).await;
            }
            // Handle the case where the aspect ratio is not square
            else {
                info!("Video is not square");

                bot.send_message(message.chat.id, "*Ширина видео не равняется высоте\\!\nВам возможно придётся уменьшить размер видео в редакторе в телеграме*")
                    .reply_parameters(ReplyParameters::new(message.id))
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?;
                bot.send_message(message.chat.id, "*Подождите\\.\\.\\.*")
                    .reply_parameters(ReplyParameters::new(message.id))
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?;

                let download_result = download_video_as_file(&bot, video).await;

                return handle_non_square_video_download(bot, message, download_result).await;
            }
        }
        // Handle the case where the duration is longer than max duration supported by telegram api
        else {
            info!("Video duration exceeds 60 seconds, not sending as video note.");
            bot.send_message(
                message.chat.id,
                "*Длительность видео достигла лимита в 1 минуту\\!*",
            )
            .reply_parameters(ReplyParameters::new(message.id))
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

            return Ok(());
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting bot...");
    let bot = Bot::from_env();
    let get_me = bot.get_me().await.expect("Failed to get bot info");

    info!(
        "Succesfully logged in @{} ID: {}",
        get_me.username(),
        get_me.id,
    );

    // Message handler for commands
    let command_handler = Update::filter_message()
        .filter_command::<Command>()
        .branch(endpoint(start_help_handler));

    // Message handler for video documents
    let video_handler = Update::filter_message()
        .filter(|message: Message| message.video().is_some())
        .branch(endpoint(video_handler)); // Wrap video_handler with endpoint

    let handler = dptree::entry()
        .branch(command_handler)
        .branch(video_handler);

    // Start the dispatcher
    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .default_handler(|upd| async move {
            warn!("Unhandled update: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .build()
        .dispatch()
        .await;

    Ok(())
}

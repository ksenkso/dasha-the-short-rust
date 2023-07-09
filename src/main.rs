use std::error::Error;
use std::str::FromStr;

use rustube::{Stream, Video};
use rustube::url::Url;
use teloxide::Bot;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::dptree;
use teloxide::prelude::Dispatcher;
use teloxide::requests::Requester;
use teloxide::types::{Update, Message, InputFile};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let bot = Bot::from_env();
    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler));
    // teloxide::repl(bot, message_handler);

    Dispatcher::builder(bot, handler)
        .default_handler(|_| async {})
        .build()
        .dispatch()
        .await;
}

async fn get_best_quality_stream<'a>(video: &'a Video) -> Option<&'a Stream> {
    return video
        .streams()
        .iter()
        .filter(|stream| stream.includes_video_track && stream.includes_audio_track)
        .max_by_key(|stream| stream.quality_label);
}

async fn message_handler(bot: Bot, message: Message) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat = &message.chat;
    let chat_id = chat.id;
    let text = message.text();

    log::info!("Handling a message");

    if let Some(video_url) = get_youtube_url(text) {

        log::info!("Got youtube url: {}", video_url);

        let video = rustube::Video::from_url(&video_url).await;
        match video {
            Ok(video) => {
                match get_best_quality_stream(&video).await {
                    Some(stream) => {
                        log::info!("Starting the download: {}", stream.signature_cipher.url);

                        let download = stream.download_to_dir("/tmp").await;
                        let video_id = stream.video_details.video_id.to_string();

                        match download {
                            Ok(path) => {
                                log::info!("Downloaded to {}", path.to_str().unwrap_or("NO_PATH"));
                                let video_file = InputFile::file(path);
                                let request = bot.send_video(chat_id, video_file).await;
                    
                                match request {
                                    Ok(_) => {
                                        log::info!("Video sent: {}", video_id);
                                    },
                                    Err(error) => {
                                        log::error!("Error while sending the video: {}\nVideo id: {}", error, video_id);
                                    }
                                }
                            },
                            Err(error) => {
                                log::error!("Error while downloading the video: {}\nVideo id: {}", error, video_id);
                            }
                        }
                    },
                    None => {
                        log::error!("No video stream");
                    }
                }
            },
            Err(_) => {
                log::error!("Error while parsing video info");
            }
        }

        bot.send_message(chat_id, "this is a youtube url").await?;
    } else {
        log::info!("Not a youtube url");
    }
    Ok(())
}

fn get_youtube_url<'a>(text: Option<&'a str>) -> Option<Url> {
    match text {
        Some(t) => {
            if t.starts_with("https://") && t.contains("youtube.com/") {
                let url = Url::from_str(t);
                if let Ok(video_url) = url {
                    return Some(video_url);
                }
                return None;
            }
            return None;
        },
        None => None,
    }
}

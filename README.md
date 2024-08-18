# Viesbot
`viesbot` is a [teloxide](https://github.com/teloxide/teloxide) based Telegram bot to convert video to video message **(aka circle)**

## Installation
You can either compile this program by yourself:
```
$ git clone https://github.com/flaclover/viesbot
$ cargo build --release
```
### Or
Download binary from [release page](https://github.com/flaclover/viesbot)

### Dependencies
For proper usage you need ffmpeg (for all platforms) and OpenSSL (if you don't use windows) installed (at least ffmpeg 2.0  required)

## How to use
Create `temp` directory in current directory

```
$ TELOXIDE_TOKEN=YOUR_TELEGRAM_BOT_TOKEN ./viesbot
```
replace `./viesbot` with actual path to viesbot binary, \
and replace `YOUR_TELEGRAM_BOT_TOKEN` with actual token that you can get from [@botfather](https://t.me/botfather) \
To stop bot press Ctrl+c in your terminal

## How to use this telegram bot
Use /help or /start to display help message. \
Send video to bot to get Video message **(aka circle)** \
If the weight and height of the video are not equal, you should crop the area where you want to place the video message.

## Support
Monero:`8BnPQDMFf8yBUg33ZuEVyFHmgYUufcRyaUpRXDXvmsca3HcKHt6tM3oBJULmDnwQELFx3mDkkjcyMezhnNcpqvMsU86zRHJ`


## License
This project is licensed under the BSD 3-Clause License. See the [LICENSE](LICENSE) file for details.

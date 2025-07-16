# Readeck Bot

Readeck bot for Telegram

## Setup

Before program execution set the `BOT_TOKEN` environment variable with the token provided by BotFather and the `API_URL` to the url of your Readeck instance (for example `https://read.example.com`).\
By default, the `LOG_LEVEL` environment variable is set to `INFO`.\
If you want extremly verbose and detailed output, you can set it to `TRACE` or `DEBUG`.\
If you want less output you can set it to `WARN`, or for even less output, to `ERROR`, although this should be avoided.

The `PRETTY_PRINT` environment variable is also available: if set the reply will inform the user with the estimated reading time and the title. Note that this information is not always available.

Telegram users allowed to use the bot are defined in the `users.json` file, which must be structured like this

```json
{
    "users": {
        "TELEGRAM_USER_ID": {
            "api_token": "READECK_TOKEN",
            "bot_marked": true
        }
    }
}

```

The `bot_marked` field specify if the article added by the bot should be labelled as `readeck-bot`.

If you want to use a different name or path for the domains file, set the USERS_FILE environment variable to the desired path.

A docker compose example file is available [here](https://raw.githubusercontent.com/matteof04/readeckbot/refs/heads/main/compose.yaml).

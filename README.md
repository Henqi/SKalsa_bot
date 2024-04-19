# SKalsa Telegram Bot

My feeble attempt at creating a working Telegram bot.

This telegram bot checks if Arena Center Hakaniemi (aka Hakis) has an available shift at a specific time and posts the data as response

## How to use bot

1. Clone the repo

    ```bash
    git clone git@github.com:Henqi/SKalsa_bot.git
    ```

2. Install poetry (https://python-poetry.org/docs/)
3. Setup .env file with secret telegram bot APIKEY & LOGFILE_PATH
4. Install dependencies

    ```bash
    poetry install
    ```

5. Run skalsa_bot.py

    ```bash
    poetry run python src/skalsa_bot.py
    ```

6. Hope it works 🤞

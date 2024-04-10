import datetime
import logging
import os

from datetime import date
from logging.handlers import TimedRotatingFileHandler

import requests

from dotenv import load_dotenv
from telegram import Update
from telegram.ext import ApplicationBuilder, CommandHandler, ContextTypes

load_dotenv()
api_key = os.getenv("API_KEY")
logfile_path = os.getenv("LOGFILE_PATH")

logging.basicConfig(
    handlers=[TimedRotatingFileHandler(logfile_path, when="midnight", backupCount=30)],
    level=logging.DEBUG,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    datefmt="%Y-%m-%dT%H:%M:%S",
)
logging.getLogger("hpack").setLevel("INFO")


def get_next_same_weekday(date, weekday):
    # takes as input a date and a weekday.
    # If the given date is the weekday in question, returns the given date.
    # Otherwise looks for the next weekday in question.
    #  1 = monday
    #  2 = tuesday
    #  3 = wednesday
    #  4 = thursday
    #  5 = friday
    #  6 = saturday
    #  7 = sunday

    if weekday <= 0 or weekday > 7:
        raise ValueError("Input value must be between 0 and 7 (inclusive)")
    else:
        days = (weekday - date.isoweekday() + 7) % 7
        return date + datetime.timedelta(days=days)


async def start(update: Update, context: ContextTypes.DEFAULT_TYPE):
    await context.bot.send_message(
        chat_id=update.effective_chat.id,
        text="Hei! Olen KalsaBot ja haen tietoja Kalsan kotiluolasta Hakiksesta!",
    )


async def hakis(update: Update, context: ContextTypes.DEFAULT_TYPE):
    today = date.today()
    day = get_next_same_weekday(today, 3)
    day_as_string = day.strftime("%Y-%m-%d")
    hour = "18"
    ismultibooking = "false"
    branch_id = "2b325906-5b7a-11e9-8370-fa163e3c66dd"
    group_id = "a17ccc08-838a-11e9-8fd9-fa163e3c66dd"
    product_id = "59305e30-8b49-11e9-800b-fa163e3c66dd"
    user_id = "d7c92d04-807b-11e9-b480-fa163e3c66dd"  # kenttä2

    day_url = (
        "https://avoinna24.fi/api/slot?filter[ismultibooking]="
        + ismultibooking
        + "&filter[branch_id]="
        + branch_id
        + "&filter[group_id]="
        + group_id
        + "&filter[product_id]="
        + product_id
        + "&filter[user_id]="
        + user_id
        + "&filter[date]="
        + day_as_string
        + "&filter[start]="
        + day_as_string
        + "&filter[end]="
        + day_as_string
    )
    headers = {"X-Subdomain": "arenacenter"}

    reservation_data = requests.get(day_url, headers=headers)
    reservation_json = reservation_data.json()

    if len(reservation_json["data"]) > 0:
        for slot in reservation_json["data"]:
            if day_as_string + " " + hour in slot["attributes"]["endtime"]:
                result = (
                    "Päivälle " + day_as_string + " on vapaana vuoro joka loppuu tunnilla " + hour
                )
                break
        result = "Päivälle " + day_as_string + " EI OLE vapaata vuoroa joka loppuu tunnilla " + hour

    else:
        result = (
            "Päivälle " + day_as_string + " ei löytynyt yhtään vapaata vuoroa / dataa ei löytynyt"
        )

    await context.bot.send_message(chat_id=update.effective_chat.id, text=result)


async def delsu(update: Update, context: ContextTypes.DEFAULT_TYPE):
    today = date.today()
    day = get_next_same_weekday(today, 2)
    day_as_string = day.strftime("%Y-%m-%d")
    hour = "19"
    ismultibooking = "false"
    branch_id = "2b325906-5b7a-11e9-8370-fa163e3c66dd"
    group_id = "a17ccc08-838a-11e9-8fd9-fa163e3c66dd"
    product_id = "59305e30-8b49-11e9-800b-fa163e3c66dd"
    user_id = "ea8b1cf4-807b-11e9-93b7-fa163e3c66dd"  # kenttä3

    day_url = (
        "https://avoinna24.fi/api/slot?filter[ismultibooking]="
        + ismultibooking
        + "&filter[branch_id]="
        + branch_id
        + "&filter[group_id]="
        + group_id
        + "&filter[product_id]="
        + product_id
        + "&filter[user_id]="
        + user_id
        + "&filter[date]="
        + day_as_string
        + "&filter[start]="
        + day_as_string
        + "&filter[end]="
        + day_as_string
    )
    headers = {"X-Subdomain": "arenacenter"}

    reservation_data = requests.get(day_url, headers=headers)
    reservation_json = reservation_data.json()

    if len(reservation_json["data"]) > 0:
        for slot in reservation_json["data"]:
            if day_as_string + " " + hour in slot["attributes"]["endtime"]:
                result = (
                    "Päivälle " + day_as_string + " on vapaana vuoro joka loppuu tunnilla " + hour
                )
                break
        result = "Päivälle " + day_as_string + " EI OLE vapaata vuoroa joka loppuu tunnilla " + hour

    else:
        result = (
            "Päivälle " + day_as_string + " ei löytynyt yhtään vapaata vuoroa / dataa ei löytynyt"
        )

    await context.bot.send_message(chat_id=update.effective_chat.id, text=result)


if __name__ == "__main__":
    application = ApplicationBuilder().token(api_key).build()

    start_handler = CommandHandler("start", start)
    application.add_handler(start_handler)

    hakis_handler = CommandHandler("hakis", hakis)
    application.add_handler(hakis_handler)

    delsu_handler = CommandHandler("delsu", delsu)
    application.add_handler(delsu_handler)

    application.run_polling()

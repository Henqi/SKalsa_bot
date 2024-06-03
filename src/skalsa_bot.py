import datetime
import logging
import os

from datetime import date, datetime as dt
from logging.handlers import TimedRotatingFileHandler
from zoneinfo import ZoneInfo

from dotenv import load_dotenv
from telegram import Update
from telegram.ext import ApplicationBuilder, CommandHandler, ContextTypes

import requests

# Define Helsinki timezone
helsinki_tz = ZoneInfo("Europe/Helsinki")

# Define UTC timezone
utc_tz = ZoneInfo("UTC")

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


def get_next_same_weekday(date: datetime.date, weekday: int):
    """Returns the date of the next occurrence of the selected weekday.
    If the given date is the selected weekday, returns the given date.
    Otherwise looks for the next occurrence of the selected weekday.
     1 = monday
     2 = tuesday
     3 = wednesday
     4 = thursday
     5 = friday
     6 = saturday
     7 = sunday
    """
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
    day_as_string = day.isoformat()
    hour = "18"
    ismultibooking = "false"
    branch_id = "2b325906-5b7a-11e9-8370-fa163e3c66dd"
    group_id = "a17ccc08-838a-11e9-8fd9-fa163e3c66dd"
    product_id = "59305e30-8b49-11e9-800b-fa163e3c66dd"
    user_id = "d7c92d04-807b-11e9-b480-fa163e3c66dd"  # kenttä2

    result = check_slot_availability(
        branch_id, group_id, product_id, user_id, ismultibooking, hour, day_as_string
    )

    await context.bot.send_message(chat_id=update.effective_chat.id, text=result)


async def delsu(update: Update, context: ContextTypes.DEFAULT_TYPE):
    today = date.today()
    day = get_next_same_weekday(today, 2)
    day_as_string = day.isoformat()
    hour = "19"
    ismultibooking = "false"
    branch_id = "2b325906-5b7a-11e9-8370-fa163e3c66dd"
    group_id = "a17ccc08-838a-11e9-8fd9-fa163e3c66dd"
    product_id = "59305e30-8b49-11e9-800b-fa163e3c66dd"
    user_id = "ea8b1cf4-807b-11e9-93b7-fa163e3c66dd"  # kenttä3

    result = check_slot_availability(
        branch_id, group_id, product_id, user_id, ismultibooking, hour, day_as_string
    )

    await context.bot.send_message(chat_id=update.effective_chat.id, text=result)


def check_slot_availability(
    branch_id, group_id, product_id, user_id, ismultibooking, hour, day_as_string
):
    day_url = (
        f"https://avoinna24.fi/api/slot?filter[ismultibooking]={ismultibooking}"
        f"&filter[branch_id]={branch_id}"
        f"&filter[group_id]={group_id}"
        f"&filter[product_id]={product_id}"
        f"&filter[user_id]={user_id}"
        f"&filter[date]={day_as_string}"
        f"&filter[start]={day_as_string}"
        f"&filter[end]={day_as_string}"
    )
    headers = {"X-Subdomain": "arenacenter"}

    reservation_data = requests.get(day_url, headers=headers)
    reservation_json = reservation_data.json()

    if len(reservation_json["data"]) > 0:
        for slot in reservation_json["data"]:

            date_str = str(slot["attributes"]["endtime"])
            date_isoformat = dt.fromisoformat(date_str)
            date_isoformat = date_isoformat.replace(tzinfo=utc_tz)
            date_isoformat = date_isoformat.astimezone(helsinki_tz)
            date_isoformat_str = date_isoformat.strftime("%Y-%m-%d")

            if date_isoformat_str == day_as_string and date_isoformat.hour == int(hour):
                result = f"Päivälle {day_as_string} on vapaana vuoro joka loppuu tunnilla {hour}"
                return result
        return f"Päivälle {day_as_string} EI OLE vapaata vuoroa joka loppuu tunnilla {hour}"
    else:
        return f"Päivälle {day_as_string} ei löytynyt yhtään vapaata vuoroa / dataa ei löytynyt"


if __name__ == "__main__":
    application = ApplicationBuilder().token(api_key).build()

    start_handler = CommandHandler("start", start)
    application.add_handler(start_handler)

    hakis_handler = CommandHandler("hakis", hakis)
    application.add_handler(hakis_handler)

    delsu_handler = CommandHandler("delsu", delsu)
    application.add_handler(delsu_handler)

    application.run_polling()

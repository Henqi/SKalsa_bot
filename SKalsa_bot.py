import logging
from telegram import Update
from telegram.ext import ApplicationBuilder, ContextTypes, CommandHandler

from dotenv import load_dotenv
import os
import requests
from datetime import date
import datetime

load_dotenv()
api_key = os.getenv("API_KEY")

logging.basicConfig(
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    level=logging.INFO
)

def get_next_same_weekday(date, day):
    days = (day - date.isoweekday() + 7) % 7
    return date + datetime.timedelta(days=days)

async def start(update: Update, context: ContextTypes.DEFAULT_TYPE):
    await context.bot.send_message(chat_id=update.effective_chat.id, text="Hei! Olen KalsaBot ja haen tietoja Kalsan kotiluolasta Hakiksesta!")

async def hakis(update: Update, context: ContextTypes.DEFAULT_TYPE):
    today = date.today()
    day = get_next_same_weekday(today, 3)
    day_as_string = day.strftime('%Y-%m-%d')
    hour='18'
    ismultibooking = 'false'
    branch_id = '2b325906-5b7a-11e9-8370-fa163e3c66dd'
    group_id = 'a17ccc08-838a-11e9-8fd9-fa163e3c66dd'
    product_id = '59305e30-8b49-11e9-800b-fa163e3c66dd'


    day_url = 'https://avoinna24.fi/api/slot?filter[ismultibooking]=' + ismultibooking + '&filter[branch_id]=' + branch_id + '&filter[group_id]=' + group_id + '&filter[product_id]=' + product_id + '&filter[date]=' + day_as_string + '&filter[start]=' + day_as_string + '&filter[end]=' + day_as_string
    headers = {'X-Subdomain': 'arenacenter'}
    
    reservation_data = requests.get(day_url, headers=headers)
    reservation_json = reservation_data.json()

    if len(reservation_json['data']) > 0:
        for slot in reservation_json['data']:
            if day_as_string + ' ' + hour in slot['attributes']['endtime']:
                result = "Päivälle " + day_as_string + " on vapaana vuoro joka loppuu tunnilla " + hour 
                break
        result = "Päivälle " + day_as_string + " EI OLE vapaata vuoroa joka loppuu tunnilla " + hour 
    
    else:
        result = "Vuorot eivät ole vielä varattavissa koska dataa ei löytynyt"

    await context.bot.send_message(chat_id=update.effective_chat.id, text=result)


if __name__ == '__main__':
    application = ApplicationBuilder().token(api_key).build()
    
    start_handler = CommandHandler('start', start)
    application.add_handler(start_handler)

    hakis_handler = CommandHandler('hakis', hakis)
    application.add_handler(hakis_handler)
    
    application.run_polling()
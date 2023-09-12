import requests
import discord
import asyncio
import json

# Konfiguration
WEB_URL = "https://blackfur.oops.wtf/get/CLASSNAME"
DISCORD_TOKEN = "TOKEN_HERE"
MAIN_CHANNEL_ID = 0 # Channel id where to send main message
ERROR_CHANNEL_ID = 0 # Channel id where to send error messages
INTERVAL_SECONDS = 300

# Discord-Bot einrichten
intents = discord.Intents.default()
intents.typing = False
intents.presences = False
bot = discord.Client(intents=intents)

def get_data():
    contents = requests.get(WEB_URL).text
    return json.loads(contents)

# Die Hauptfunktion, die die Änderungen überwacht und bei Bedarf Nachrichten an Discord sendet
@bot.event
async def on_ready():
    print(f'Eingeloggt als {bot.user.name}')
    previous_data = None
    previous_message = None
    while True:
        try:
            current_data = get_data()
            if current_data is not None and current_data != previous_data:
                # Eine Änderung wurde festgestellt
                has_elements = False
                discord_channel = bot.get_channel(MAIN_CHANNEL_ID)
                if discord_channel:
                    embed = discord.Embed(
                        type="rich", title="Vertretungsplan", description="Es gab eine Änderung\n\u200B")
                    for info in current_data["infos"]:
                        embed.add_field(name=f'**Zusätzliche Informationen für {info["day"]}**', value=f'{info["info"]}\n\u200B')
                        has_elements = True
                    for value in current_data["data"]:
                        embed.add_field(name=f'**{value["day"]}**',
                                        value=f'''
Stunde: {value["lesson"]}
Fach: {value["subject"]}
Raum: {value["room"]}
Lehrer: {value["teachers"]}
Art: {value["info"]}
Information: {value["notes"]}
\u200B
                                            ''',
                                            inline=True)
                        has_elements = True
                    if not has_elements:
                        embed.description = "Es gibt keine Änderungen für die nächsten Tage."
                    current_message = await discord_channel.send(embed=embed)
                if previous_message is not None:
                    await previous_message.delete()
                previous_message = current_message
                previous_data = current_data
            else:
                print("Keine Änderung auf der Webseite gefunden.")
        except:
            try:
                error_channel = bot.fetch_channel(ERROR_CHANNEL_ID)
                if error_channel:
                    await error_channel.send(f"Es gab einen fehler bei der Abtrage der API: {current_data['error']}")
            except:
                print("Es konnte keine Fehlernachricht gesendet werden.")
        await asyncio.sleep(INTERVAL_SECONDS) 

if __name__ == "__main__":
    bot.run(DISCORD_TOKEN)
import datetime
import random
import re
from rori import DBManager, Module

class Database(DBManager):
    def select_message_from_today(self, author):
        dbcur = self.conn.cursor()
        current_day = str(datetime.datetime.now()).split(' ')[0]
        today_messages = "SELECT body From History Where author_ring_id=\"{0}\" AND tm>= Datetime('{1}');".format(author, current_day)
        return dbcur.execute(today_messages).fetchall()

class Module(Module):
    def process(self, interaction):
        alreadySeen = False
        nbSeen = 0
        for message in Database().select_message_from_today(interaction.author_ring_id):
            m = re.findall(r"^(salut|bonjour|bonsoir|hei|hi|hello|yo|o/)( rori| ?!?)$", message[0], flags=re.IGNORECASE)
            if len(m) > 0:
                nbSeen += 1
                if nbSeen > 1:
                    alreadySeen = True
                    break
        if alreadySeen:
            randomstr = random.choice(["already", "already2", ""])
            string_to_say = self.rori.get_localized_sentence(randomstr, self.sentences)
            res = self.rori.send_for_best_client("text", interaction.author_ring_id, string_to_say)
        else:
            randomstr = random.choice(["salut", "bonjour", "longtime", "o/"])
            string_to_say = self.rori.get_localized_sentence(randomstr, self.sentences)
            res = self.rori.send_for_best_client("text", interaction.author_ring_id, string_to_say)
        # TODO change emotions self.rori.go_to(emotion, direction)
        self.stop_processing = True

import datetime
import random
from rori import DBManager, Interaction, Module

class Database(DBManager):
    def store_data(self, interaction):
        '''Store an interaction into the database'''
        dbcur = self.conn.cursor()
        isMessageTableRequest = "SELECT * FROM sqlite_master WHERE name ='History' and type='table';"
        dbcur.execute(isMessageTableRequest)
        result = dbcur.fetchone()
        if not result:
            createTableRequest = "CREATE TABLE History(id INTEGER PRIMARY KEY ASC, author_ring_id TEXT, body TEXT, tm DATETIME);"
            dbcur.execute(createTableRequest)
            self.conn.commit()
        time = datetime.datetime.strptime(interaction.time[:19], '%Y-%m-%dT%H:%M:%S')
        tz = interaction.time[19:]
        if len(tz) > 4:
            h = int(tz[1:3])
            m = int(tz[4:])
            if tz[0] == '+':
                time -= datetime.timedelta(hours=h, minutes=m)
            else:
                time += datetime.timedelta(hours=h, minutes=m)
        addMessageRequest = "INSERT INTO History(author_ring_id, body, tm) VALUES(\"{0}\",\"{1}\",\"{2}\")".format(interaction.author_ring_id, interaction.body, str(time))
        dbcur.execute(addMessageRequest)
        self.conn.commit()

class Module(Module):
    def process(self, interaction):
        Database().store_data(interaction)

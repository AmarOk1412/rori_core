from rori import DBManager, Interaction, Module
import datetime
import random

class Database(DBManager):
    def store_data(self, interaction):
        dbcur = self.conn.cursor()
        isMessageTableRequest = "SELECT * FROM sqlite_master WHERE name ='History' and type='table';"
        dbcur.execute(isMessageTableRequest)
        result = dbcur.fetchone()
        if not result:
            createTableRequest = "CREATE TABLE History(id INTEGER PRIMARY KEY ASC, author_ring_id TEXT, body TEXT, tm DATETIME);"
            dbcur.execute(createTableRequest)
            self.conn.commit()
        # TODO use interaction time instead of datetime
        addMessageRequest = "INSERT INTO History(author_ring_id, body, tm) VALUES(\"{0}\",\"{1}\",\"{2}\")".format(interaction.author_ring_id, interaction.body, str(datetime.datetime.now()))
        dbcur.execute(addMessageRequest)
        self.conn.commit()

class Module(Module):
    def process(self, interaction):
        Database().store_data(interaction)

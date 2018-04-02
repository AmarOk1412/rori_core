import sqlite3

class DBManager:
    def __init__(self):
        self.conn=sqlite3.connect('rori.db')

    def __del__(self):
        self.conn.close()

import sqlite3

conn = sqlite3.connect('rori.db')
c = conn.cursor()

# talk/history
print('add history module')
arguments = '("history", 0, 1, "text", ".*", "history")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/hello_world
print('add hello_world module')
arguments = '("hello_world", 1, 1, "text", "^(salut|bonjour|bonsoir|hei|hi|hello|yo|o/)( rori| ?!?)$", "talk/hello_world")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

conn.commit()

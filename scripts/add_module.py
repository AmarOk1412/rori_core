import sqlite3

name = input('New module: ')
priority = input('With priority: ')
is_enabled = input('enabled (Y/n): ').lower() != 'n'
enabled = 1 if is_enabled else 0
typem = input('With type: ')
condition = input('With condition: ')
path = input('With path: ')

# TODO metadatas

conn = sqlite3.connect('rori.db')
c = conn.cursor()
arguments = '("' + name + '", ' + str(priority) + ', ' + str(enabled) + ', "'
arguments += typem + '", "' + condition + '", "' + path + '")'
print('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)
print(c.lastrowid)
conn.commit()

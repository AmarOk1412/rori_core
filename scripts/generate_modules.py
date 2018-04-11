import sqlite3

conn = sqlite3.connect('rori.db')
c = conn.cursor()

# Clean modules
c.execute('DELETE FROM modules WHERE 1=1')

# talk/history
print('add history module')
arguments = '("history", 0, 1, "text", ".*", "history")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/hello_world
print('add hello_world module')
arguments = '("hello_world", 1, 1, "text", "^(salut|bonjour|bonsoir|hei|hi|hello|yo|o/)( rori| ?!?)$", "talk/hello_world")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/goodbye_world
print('add goodbye_world module')
arguments = '("goodbye_world", 1, 1, "text", "^(au(.?)revoir|(à|a) la prochaine|bonne soir(ée|ee)|good( |-)bye|bye|j.y.vais)", "talk/goodbye_world")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/age
print('add age module')
arguments = '("age", 1, 1, "text", "^(quel es(t) ton.{0,20}(â|a)ge|how old are you|quel.{0,20}(â|a)ge.{0,20}(tu)|quan.{0,10}(existe|tu né|(t\'|tu).{0,30}cr(éé|ee))|Since when.{0,100}exist|when.{0,100}create)", "talk/age")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/license
print('add license module')
arguments = '("license", 1, 1, "text", "puis.je.{0,100}(source|code)|sous.quelle.licence|(where |a?)can i read.{0,100}code|are you.{0,20}(free|under.{0,100}license)", "talk/license")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

conn.commit()

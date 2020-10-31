import sqlite3

conn = sqlite3.connect('rori.db')
c = conn.cursor()

# Clean modules
c.execute('CREATE TABLE IF NOT EXISTS modules (id INTEGER PRIMARY KEY, name TEXT, priority INTEGER, enabled BOOLEAN, type TEXT, condition TEXT, path TEXT)')
c.execute('DELETE FROM modules WHERE 1=1')

# talk/history
print('add history module')
arguments = '("history", 0, 1, "plain/text", ".*", "history")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/hello_world
print('add hello_world module')
arguments = '("hello_world", 1, 1, "plain/text", "^(salut|bonjour|bonsoir|hei|hi|hello|yo|o/)( rori| ?!?)$", "talk/hello_world")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/thank
print('add thank module')
arguments = '("thank", 1, 1, "plain/text", "^(merci|thx|thanks|thank you)( rori| ?!?)$", "talk/thank")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/goodbye_world
print('add goodbye_world module')
arguments = '("goodbye_world", 1, 1, "plain/text", "^(au(.?)revoir|(à|a) la prochaine|bonne soir(ée|ee)|good( |-)bye|bye|j.y.vais)", "talk/goodbye_world")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/age
print('add age module')
arguments = '("age", 1, 1, "plain/text", "^(quel es(t) ton.{0,20}(â|a)ge|how old are you|quel.{0,20}(â|a)ge.{0,20}(tu)|quan.{0,10}(existe|tu né|(t\'|tu).{0,30}cr(éé|ee))|Since when.{0,100}exist|when.{0,100}create)", "talk/age")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/license
print('add license module')
arguments = '("license", 1, 1, "plain/text", "puis.je.{0,100}(source|code)|sous.quelle.licence|(where |a?)can i read.{0,100}code|are you.{0,20}(free|under.{0,100}license)", "talk/license")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/creator
print('add creator module')
arguments = '("creator", 1, 1, "plain/text", "^(pr(é|e)sente|qui).{0,30}(amarok|ton cr(é|e)ateur|t\'a(s) cr(éé|ee))|who.{0,100}(create|is amarok|programmer|creator)", "talk/creator")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/who
print('add who module')
arguments = '("who", 2, 1, "plain/text", "who.is|qui.es(t)", "talk/who")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/name
print('add name module')
arguments = '("name", 2, 1, "plain/text", "^(quel est? ton.{0,30}nom.{0,40})|qui e(s|t).{0,4}(tu|vous)|o(u|ù)vien(s|t).ton.{0,20}nom|say.your.name|what.is.your.name|who.are.you|why.rori|rori.{0,100}come.from", "talk/name")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/humor
print('add humor module')
arguments = '("humor", 2, 1, "plain/text", "(c|ç)a.va.{0,10}$|tu.{0,30}va(s| ?).{0,30}(bien|mal|comment|bof)|are you ok|comment.{0,30}va(s| ?)|how are you", "talk/humor")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/sing
print('add sing module')
arguments = '("sing", 2, 1, "plain/text", "^(tu peux| ?)chante(r| ?)|sing", "talk/sing")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/alarm
print('add alarm module')
arguments = '("alarm", 2, 1, "plain/text", "(wake|veille).{0,100}(in|at|dans|à|a).([0-9]+)(:|h|.*)([0-9]*)", "talk/alarm")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# talk/uptime
print('add uptime module')
arguments = '("uptime", 2, 1, "plain/text", "uptime|since.when.{0,20}up", "talk/uptime")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# command/blackscreen
print('add blackscreen module')
arguments = '("blackscreen", 2, 1, "plain/text", "^((é|e)cran.noir|black.?screen|(go to | ?)sleep|(vas | ?)dor(s|t|mir))", "command/blackscreen")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# command/mutesound
print('add mutesound module')
arguments = '("mutesound", 2, 1, "plain/text", "^(sourdine|muet|mute|coupe le son|no sound please)", "command/mutesound")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# command/enablesound
print('add enablesound module')
arguments = '("enablesound", 2, 1, "plain/text", "^(remet le |a?)son$|on.{0,20}sound|sound.{0,20}on|umute", "command/enablesound")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# command/lock
print('add lock module')
arguments = '("lock", 2, 1, "plain/text", "(ver(r| ?)ouil(l| ?)(e (l.ordi|le pc|l\'(é|e)cran|toi)|age)|bloque.toi)|^lock", "command/lock")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# command/meteo
print('add meteo module')
arguments = '("meteo", 2, 1, "plain/text", "(weather|meteo)", "command/meteo")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# command/news
print('add news module')
arguments = '("news", 2, 1, "plain/text", "open.+news", "command/news")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# command/news
print('add feed module')
arguments = '("feed", 2, 1, "plain/text", "$", "command/feed")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# music/music_start
print('add music_start module')
arguments = '("music_start", 1, 1, "plain/text", "^(musi(c|que) ?!?)|((play|lance|joue).{1,30}(musi(c|que) ?!?))$", "music/start")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# music/music_stop
print('add music_stop module')
arguments = '("music_stop", 1, 1, "plain/text", "^stop.{1,8}musi(c|que)", "music/stop")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# music/music_pause
print('add music_pause module')
arguments = '("music_pause", 1, 1, "plain/text", "^pause", "music/pause")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# music/music_next
print('add music_next module')
arguments = '("music_next", 1, 1, "plain/text", "^next.{1,8}musi(c|que)", "music/next")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

# music/music_previous
print('add music_previous module')
arguments = '("music_previous", 1, 1, "plain/text", "^previous.{1,8}musi(c|que)", "music/previous")'
c.execute('INSERT INTO modules (name, priority, enabled, type, condition, path) VALUES' + arguments)

conn.commit()

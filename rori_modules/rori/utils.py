import sqlite3

class DBManager:
    def __init__(self):
        self.conn=sqlite3.connect('rori.db')

    def __del__(self):
        self.conn.close()

class EmotionsManager:
    def __init__(self):
        self.conn=sqlite3.connect('rori.db')

    def get_emotions(self, ring_id):
        # get linked username
        username = self.conn.execute("SELECT username FROM devices WHERE ring_id=\"{ring_id}\"").fetchone()
        if username == None:
            username = ""
        # Get current emotions for this user
        result = self.conn.execute('SELECT love, joy, surprise, anger, sadness, fear FROM emotions WHERE username=\"' + username + '\"').fetchone()
        if result == None:
            # Non existing, init emotions
            self.conn.execute('INSERT INTO Emotions (username, love, joy, surprise, anger, sadness, fear) VALUES (\"' + username + '\", 50, 50, 50, 50, 50, 50)').fetchone()
            self.conn.commit()
            return (50, 50, 50, 50, 50, 50)
        return result

    def go_to_emotion(self, ring_id, love = None, joy = None, surprise = None, anger = None, sadness = None, fear = None, delta = 1):
        username = self.conn.execute("SELECT username FROM devices WHERE ring_id=\"{ring_id}\"").fetchone()
        if username == None:
            username = ""
        cemotions = self.get_emotions(ring_id)
        clove, cjoy, csurprise, canger, csadness, cfear = zip(cemotions)
        clove, cjoy, csurprise, canger, csadness, cfear = clove[0], cjoy[0], csurprise[0], canger[0], csadness[0], cfear[0]
        if love != None and love != clove:
            clove = min(clove + delta, love) if love > clove else max(clove - delta, love)
        if joy != None and joy != cjoy:
            cjoy = min(cjoy + delta, joy) if joy > cjoy else max(cjoy - delta, joy)
        if surprise != None and surprise != csurprise:
            csurprise = min(csurprise + delta, surprise) if surprise > csurprise else max(csurprise - delta, surprise)
        if anger != None and anger != canger:
            canger = min(canger + delta, anger) if anger > canger else max(canger - delta, anger)
        if sadness != None and sadness != csadness:
            csadness = min(csadness + delta, sadness) if sadness > csadness else max(csadness - delta, sadness)
        if fear != None and fear != cfear:
            cfear = min(cfear + delta, fear) if fear > cfear else max(cfear - delta, fear)
        self.conn.execute('UPDATE emotions SET love=' + clove + ', joy=' + cjoy + ', surprise=' + csurprise + ', anger=' + canger + ', sadness=' + csadness + ', fear=' + cfear + ' WHERE username=\"' + username + '\"')
        self.conn.commit()
        return

    def __del__(self):
        self.conn.close()

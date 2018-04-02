import json

class Interaction:
    def __init__(self, interaction):
        json_value = json.loads(interaction)
        self.author_ring_id = json_value['author_ring_id']
        self.body = json_value['body']
        self.time = json_value['time']

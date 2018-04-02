from rori import RORI

class Module:
    def __init__(self, sentences):
        self.rori = RORI.RORI()
        self.stop_processing = False
        self.sentences = sentences

    def process(self, interaction):
        pass

    def continue_processing(self):
        return not self.stop_processing

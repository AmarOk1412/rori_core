import dbus
import json

class RORI:
    def __init__(self):
        self.lang = 'en'

    def send_for_best_client(self, datatype, from_ring_id, content):
        '''Send a mesage to the best device'''
        if len(content) == 0:
            return
        bus = dbus.SessionBus()
        configuration_mngr = bus.get_object('cx.ring.Ring', '/cx/ring/Ring/ConfigurationManager', introspect=False)
        sendTextMessage = configuration_mngr.get_dbus_method('sendTextMessage', 'cx.ring.Ring.ConfigurationManager')
        config = ''
        with open('config.json', 'r') as f:
            config = json.loads(f.read())
        sendTextMessage(config['ring_id'], from_ring_id, {'text/plain': content})
        # TODO improve this. For now send to sender
        # NOTE ignore datatype for now.

    def get_localized_sentence(self, id, data):
        '''Get translated sentence linked to a token'''
        try:
            json_data = json.loads(data)
            result = json_data[id][self.lang]
            return result
        except:
            return ""

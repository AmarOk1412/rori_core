import os.path
import importlib
from rori import Interaction, Module

def load_module(path):
    '''import a module given a path'''
    path = path.replace('/', '.')
    if path[len(path)-1] == '.':
        path = path[:-1]
    mod = importlib.import_module(path + '.module')
    return mod

def exec_module(path, interaction):
    '''execute a module given a path'''
    module_path = path
    mod = load_module(module_path)
    if path[-1] == '/':
        path = path[:-1]
    path = 'rori_modules/' + path + '/rsc.json'
    sentences = '{}'
    if os.path.isfile(path):
        with open(path, 'r') as f:
            sentences = f.read()
    m = mod.Module(sentences)
    m.process(Interaction(interaction))
    return m.continue_processing()

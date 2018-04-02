# from rori import *
import os.path

def load_module(path):
    path = path.replace("/", ".")
    if path[len(path)-1] is ".":
        path = path[:-1]
    exec("import %s.module as module" % path, globals())

def exec_module(path, interaction):
    module_path = path
    load_module(module_path)
    if path[-1] == "/":
        path = path[:-1]
    path = "rori_modules/" + path + "/rsc.json"
    sentences = "{}"
    if os.path.isfile(path):
        with open(path, 'r') as f:
            sentences = f.read()
    m = module.Module(sentences)
    m.process(interaction)
    return m.continue_processing()

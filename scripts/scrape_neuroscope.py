import neuronav as nrnv
import sys

path = sys.argv[1]

database = nrnv.Database.open(path)
database.scrape_neuroscope_model("solu-6l")
# database.scrape_neuroscope_model("gelu-1l")

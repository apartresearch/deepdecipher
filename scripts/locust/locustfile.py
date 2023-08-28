import random
from locust import FastHttpUser, HttpUser, between, task
from pyquery import PyQuery

HOST = "https://deepdecipher.org"

BASE_URL = f"{HOST}/viz"


class Model:
    def __init__(self, json):
        self.name = json["name"]

        self.url = f"/viz/{self.name}"
        self.num_layers = json["num_layers"]
        self.layer_size = json["layer_size"]

        if (
            self.name == "solu-6l-pile"
            or self.name == "gpt2-xl"
            or self.name == "gpt2-small"
        ):
            url_name = self.name
        else:
            url_name = "[model]"
        self.name_url = f"/viz/{url_name}"


def neuron_url(model: Model, layer: int, neuron: int):
    return f"{model.url}/all/{layer}/{neuron}"


class WebsiteUser(HttpUser):
    host = HOST

    wait_time = between(0.5, 30)

    models = None

    def on_start(self):
        # assume all users arrive at the index page
        with self.client.get("/api") as response:
            if response.json() is None:
                raise Exception("API did not return JSON.")

            self.models = [
                Model(model_json) for model_json in response.json()["models"]
            ]
        self.index_page()

    @task(1)
    def index_page(self):
        if self.models is None:
            return
        self.cur_content = self.client.get("/").content
        self.current_page = "index"
        self.current_model = None

    def load_model(self, model: Model):
        url = model.url
        with self.client.request("GET", url, name=f"{model.name_url}") as response:
            self.cur_content = response.content
        self.current_page = "model"
        self.current_model = model

    def load_neuron(self, model: Model, url: str):
        with self.client.request(
            "GET", url, name=f"{model.name_url}/all/[l]/[n]"
        ) as response:
            self.cur_content = response.content
        self.client.request_name = None
        self.current_page = "neuron"
        self.current_model = model

    @task(50)
    def load_page(self, url=None):
        if self.models is None:
            return
        match self.current_page:
            case "index":
                assert self.current_model == None
                self.current_model = random.choice(self.models)
                self.load_model(self.current_model)
            case "model":
                assert self.current_model != None
                layer = random.randint(0, self.current_model.num_layers - 1)
                neuron = random.randint(0, self.current_model.layer_size - 1)
                self.load_neuron(
                    self.current_model, neuron_url(self.current_model, layer, neuron)
                )
            case "neuron":
                assert self.current_model != None
                choice = random.randint(0, 100)
                pq = PyQuery(self.cur_content)
                if choice < 3:
                    self.load_model(self.current_model)
                elif choice < 40:
                    url = pq(".meta-data:nth-child(3) a")[0].attrib["href"]
                    self.load_neuron(self.current_model, url)
                else:
                    url = pq(".meta-data:nth-child(4) a")[0].attrib["href"]
                    self.load_neuron(self.current_model, url)
            case _:
                raise Exception("Unknown page type")

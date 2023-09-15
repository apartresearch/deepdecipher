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

    wait_time = between(3, 5)

    models = None

    def on_start(self):
        # assume all users arrive at the index page
        with self.client.get("/api") as response:
            if response.json() is None:
                raise Exception("API did not return JSON.")

            self.models = [
                Model(model_json) for model_json in response.json()["models"]
            ]
        exclude_models = ["gpt2-xl", "gpt2-small", "solu-6l-pile"]
        self.current_model = random.choice(
            [model for model in self.models if model.name not in exclude_models]
        )

    def load_neuron(self, model: Model, url: str):
        with self.client.request(
            "GET", url, name=f"/api/[model]/all/[l]/[n]"
        ) as response:
            self.cur_content = response.content
        self.client.request_name = None

    @task(1)
    def load_page(self, url=None):
        layer = random.randint(0, self.current_model.num_layers - 1)
        neuron = random.randint(0, self.current_model.layer_size - 1)
        url = neuron_url(self.current_model, layer, neuron)
        self.load_neuron(self.current_model, url)

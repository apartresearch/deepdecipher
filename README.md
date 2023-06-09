# NeuroNav

🦠 This repository is the open source website for NeuroNav (_NeuroScope 2.0_), a continuation of the [neuroscope.io](https://neuroscope.io/) project [Nanda, 2022].

## Data available

- NeuroScope's max activating dataset examples on 25 models
- Neuron2Graph's neuron activation model, along with the explanation power
- GPT-4's neuron activation explanation, along with the explanation power

Future data ideas:

- Which neurons have the most impact on this neuron's activation (based on weights)
- The neuron's embedding based on Neuron2Graph model
- Neuron interest variable: Variance / kurtosis of activation
- Which neurons is it connected to within the MLP layers
- Which neurons does this neuron impact the most (based on weights)
- Which tokens it passes to the residual stream (?)
- Neuron activation differences over training epochs (only available on Pythia models)
- Most correlating neurons
- Subnetwork analysis: Identification of groups of neurons that often activate together.
- Topological role: Information about the neuron's role in the overall network topology (e.g., hub, peripheral, connector, etc.) using weighted directional network summary statistics methods
- (?) Logit attribution: How much does this neuron affect the output

## Models available

| Model         | Initialisation | Activation Function | Dataset                               | Layers | Neurons per Layer | Total Neurons | Parameters    |
| ------------- | -------------- | ------------------- | ------------------------------------- | ------ | ----------------- | ------------- | ------------- |
| solu-1l       | Random         | solu                | 80% C4 (Web Text) and 20% Python Code | 1      | 2,048             | 2,048         | 3,145,728     |
| gelu-1l       | Random         | gelu                | 80% C4 (Web Text) and 20% Python Code | 1      | 2,048             | 2,048         | 3,145,728     |
| solu-2l       | Random         | solu                | 80% C4 (Web Text) and 20% Python Code | 2      | 2,048             | 4,096         | 6,291,456     |
| gelu-2l       | Random         | gelu                | 80% C4 (Web Text) and 20% Python Code | 2      | 2,048             | 4,096         | 6,291,456     |
| solu-3l       | Random         | solu                | 80% C4 (Web Text) and 20% Python Code | 3      | 2,048             | 6,144         | 9,437,184     |
| gelu-3l       | Random         | gelu                | 80% C4 (Web Text) and 20% Python Code | 3      | 2,048             | 6,144         | 9,437,184     |
| solu-4l       | Random         | solu                | 80% C4 (Web Text) and 20% Python Code | 4      | 2,048             | 8,192         | 12,582,912    |
| gelu-4l       | Random         | gelu                | 80% C4 (Web Text) and 20% Python Code | 4      | 2,048             | 8,192         | 12,582,912    |
| solu-6l       | Random         | solu                | 80% C4 (Web Text) and 20% Python Code | 6      | 3,072             | 18,432        | 42,467,328    |
| solu-8l       | Random         | solu                | 80% C4 (Web Text) and 20% Python Code | 8      | 4,096             | 32,768        | 100,663,296   |
| solu-10l      | Random         | solu                | 80% C4 (Web Text) and 20% Python Code | 10     | 5,120             | 51,200        | 196,608,000   |
| solu-12l      | Random         | solu                | 80% C4 (Web Text) and 20% Python Code | 12     | 6,144             | 73,728        | 339,738,624   |
| gpt2-small    | Random         | gelu                | Open Web Text                         | 12     | 3,072             | 36,864        | 84,934,656    |
| gpt2-medium   | Random         | gelu                | Open Web Text                         | 24     | 4,096             | 98,304        | 301,989,888   |
| gpt2-large    | Random         | gelu                | Open Web Text                         | 36     | 5,120             | 184,320       | 707,788,800   |
| gpt2-xl       | Random         | gelu                | Open Web Text                         | 48     | 6,400             | 307,200       | 1,474,560,000 |
| solu-1l-pile  | Random         | solu                | The Pile                              | 1      | 4,096             | 4,096         | 12,582,912    |
| solu-4l-pile  | Random         | solu                | The Pile                              | 4      | 2,048             | 8,192         | 12,582,912    |
| solu-2l-pile  | Random         | solu                | The Pile                              | 2      | 2,944             | 5,888         | 12,812,288    |
| solu-6l-pile  | Random         | solu                | The Pile                              | 6      | 3,072             | 18,432        | 42,467,328    |
| solu-8l-pile  | Random         | solu                | The Pile                              | 8      | 4,096             | 32,768        | 100,663,296   |
| solu-10l-pile | Random         | solu                | The Pile                              | 10     | 5,120             | 51,200        | 196,608,000   |
| pythia-70m    | Random         | gelu                | The Pile                              | 6      | 2,048             | 12,288        | 18,874,368    |
| pythia-160m   | Random         | gelu                | The Pile                              | 12     | 3,072             | 36,864        | 84,934,656    |
| pythia-350m   | Random         | gelu                | The Pile                              | 24     | 4,096             | 98,304        | 301,989,888   |

## JSON response

```
> print(request.get("https://apartresearch.com/neuronav/api/GPT-2-XL/5/2332").json())

{
    "model" : "GPT-2 XL",
    "available" : ["Neuron Graph", "GPT-4 Explanation", "Max Activating Dataset Example"],
    "layer" : 5,
    "neuron" : 2332,
    "metadata" : {
        ...
    },
    "neuroscope" : {
        ...
    },
    "neuron2graph" : {
        "explanation-score" : 0.56,
        ...
    },
    "GPT-4" {
        "explanation-score" : 0.43,
        ...
    }
}
```

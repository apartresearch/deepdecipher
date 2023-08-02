const express = require("express");
const axios = require("axios");
const path = require("path");
const app = express();
const port = 3000;

app.get("/", (req, res) => {
  res.sendFile(path.join(__dirname, "index.html"));
});

app.get("/viz/", (req, res) => {
  res.sendFile(path.join(__dirname, "index.html"));
});

app.get("/viz/:modelName", (req, res) => {
  res.sendFile(path.join(__dirname, "model.html"));
});

app.get("/viz/:modelName/:serviceName", (req, res) => {
  res.sendFile(path.join(__dirname, "model.html"));
});

app.get("/viz/:modelName/:serviceName/:layerIndex", (req, res) => {
  res.sendFile(path.join(__dirname, "layer.html"));
});

app.get(
  "/viz/:modelName/:serviceName/:layerIndex/:neuronIndex",
  (req, res) => {
    res.sendFile(path.join(__dirname, "neuron.html"));
  }
);

// Fetch js and css from the server
app.get("/js/:filename", (req, res) => {
  res.sendFile(path.join(__dirname, "js", req.params.filename));
});

app.get("/css/:filename", (req, res) => {
  res.sendFile(path.join(__dirname, "css", req.params.filename));
});

app.get(
  "/api/:modelName/:sourceName/:layerIndex/:neuronIndex",
  async (req, res) => {
    const { modelName, sourceName, layerIndex, neuronIndex } = req.params;

    try {
      const response = await axios.get(
        `http://localhost:8080/api/${modelName}/${sourceName}/${layerIndex}/${neuronIndex}`
      );
      res.json(response.data);
    } catch (err) {
      console.log(err);
      res.status(500).send("Error occurred while fetching data");
    }
  }
);

app.listen(port, () => {
  console.log(`Server listening at http://localhost:${port}`);
});

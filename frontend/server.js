const express = require("express");
const axios = require("axios");
const path = require("path");
const cors = require("cors");
const app = express();
const port = 3000;

app.get(
  "/viz/:model_name/:source_name/:layer_index/:neuron_index",
  (req, res) => {
    res.sendFile(path.join(__dirname, "neuron.html"));
  }
);

app.get("/viz/:model_name/:layer_index", (req, res) => {
  res.sendFile(path.join(__dirname, "layer.html"));
});

app.get("/viz/:model_name", (req, res) => {
  res.sendFile(path.join(__dirname, "model.html"));
});

app.get("/viz/", (req, res) => {
  res.sendFile(path.join(__dirname, "index.html"));
});

app.get("/", (req, res) => {
  res.sendFile(path.join(__dirname, "index.html"));
});

app.get("/viz/:model_name/:source_name/:layer_index", (req, res) => {
  res.sendFile(path.join(__dirname, "layer.html"));
});

// Fetch js and css from the server
app.get("/js/:filename", (req, res) => {
  res.sendFile(path.join(__dirname, "js", req.params.filename));
});

app.get("/css/:filename", (req, res) => {
  res.sendFile(path.join(__dirname, "css", req.params.filename));
});

app.get(
  "/api/:model_name/:source_name/:layer_index/:neuron_index",
  async (req, res) => {
    const { model_name, source_name, layer_index, neuron_index } = req.params;

    try {
      const response = await axios.get(
        `http://localhost:8080/api/${model_name}/${source_name}/${layer_index}/${neuron_index}`
      );
      res.json(response.data);
    } catch (err) {
      res.status(500).send("Error occurred while fetching data");
    }
  }
);

app.listen(port, () => {
  console.log(`Server listening at http://localhost:${port}`);
});

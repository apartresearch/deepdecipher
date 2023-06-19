const express = require("express");
const axios = require("axios");
const path = require("path");
const app = express();
const port = 3000;

app.get("/:model_name/:source_name/:layer_index/:neuron_index", (req, res) => {
  res.sendFile(path.join(__dirname, "index.html"));
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

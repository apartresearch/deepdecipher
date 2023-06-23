// Parse model_name, source_name, layer_index, and neuron_index from the URL
const [_, viz, model_name, source_name, layer_index] =
  location.pathname.split("/");

if (source_name != "all") {
  // Put an h1 in the #meta that says that only /all/ are supported for visualization
  const supporting = document.createElement("h1");
  supporting.innerHTML =
    source_name +
    " is not supported. Go to <a href='" +
    base_url_ui +
    base_ext_ui +
    "" +
    model_name +
    "/all/" +
    layer_index +
    "'>/all/</a> to see layer information.";
  document.getElementById("meta").appendChild(supporting);
} else {
  // Fetch data from the server
  fetch(
    `${base_url_api}${base_ext_api}${model_name}/${source_name}/${layer_index}`
  )
    .then((response) => response.json())
    .then((data) => {
      // If Neuron2Graph data is available
      if (data["neuron2graph"] != null) {
        Viz.instance().then(function (viz) {
          console.log(data);
          let svg = document.body.appendChild(
            viz.renderSVGElement(data.neuron2graph.graph)
          );
          document.getElementById("n2g").appendChild(svg);
        });
      } else {
        // Write in a div with class not_available that the data is not available
        const not_available = document.createElement("div");
        not_available.classList.add("not_available");
        not_available.textContent =
          "The Neuron to Graph data for this neuron is not available.";
        document.getElementById("n2g").appendChild(not_available);
      }

      if (data["similar"] != null) {
        for (let i = 0; i < data["similar"].length; i++) {
          const similar_neuron = document.createElement("a");
          similar_neuron.classList.add("similar_neurons");
          similar_neuron.innerHTML =
            data.similar[i].layer +
            ":" +
            data.similar[i].neuron +
            " <span data-tooltip='The similarity score to\nthis neuron'>E" +
            data.similar[i].similarity +
            "</span> <span data-tooltip='The token this neuron\nactivates the most to'>" +
            data.similar[i].token +
            "</span>";
          similar_neuron.href =
            "/viz/" +
            model_name +
            "/" +
            source_name +
            "/" +
            data.similar[i].layer +
            "/" +
            data.similar[i].neuron;
          document.getElementById("similar").appendChild(similar_neuron);
        }
      } else {
        const not_available = document.createElement("div");
        not_available.classList.add("not_available");
        not_available.textContent =
          "Similar neuron data is not available for this neuron.";
        document.getElementById("similar").appendChild(not_available);
      }

      if (data["gpt-4"] != null) {
        // If GPT-4 data is available
        const gpt4 = document.createElement("div");
        gpt4.classList.add("gpt4");
        gpt4.innerHTML = data["gpt-4"];
        document.getElementById("gpt4").appendChild(gpt4);
      } else {
        // Write in a div with class not_available that the data is not available
        const not_available = document.createElement("div");
        not_available.classList.add("not_available");
        not_available.textContent =
          "The GPT-4 data for this neuron is not available.";
        document.getElementById("gpt4").appendChild(not_available);
      }

      // Add a header for the model name
      const header = document.createElement("div");
      header.id = "header";
      header.classList.add("meta");
      document.getElementById("visualization").appendChild(header);
      // Add a header for the source_name, layer_index and neuron_index
      const meta = document.createElement("div");
      meta.id = "meta";
      meta.classList.add("meta");
      document.getElementById("visualization").appendChild(meta);
      const meta_info = document.createElement("tr");
      meta_info.innerHTML =
        "<td class='meta-data first' data-tooltip='The model name'>" +
        model_name +
        "</td><td class='meta-data' data-tooltip='The data source (all includes\n all available sources)'>" +
        source_name +
        "</td><td class='meta-data' data-tooltip='Neuron index in the layer (from 0)'>" +
        neuron_index +
        "</td><td class='meta-data' data-tooltip='Layer index in the model (from 0)'>" +
        layer_index +
        "</td>";
      document.getElementById("meta-information").appendChild(meta_info);

      if (data["model"] != null) {
        const surrounding_neurons = document.createElement("tr");
        const [layer_index_n, neuron_index_n, last_neuron, last_layer] = [
          parseInt(layer_index),
          parseInt(neuron_index),
          parseInt(data.model.neuron_per_layer - 1),
          parseInt(data.model.layer_count - 1),
        ];
        const [prev_url, next_url, layer_url, model_url] = [
          (layer_index_n == 0) & (neuron_index_n == 0)
            ? alert("This is the first neuron in the model.")
            : `${base_url_ui}${base_ext_ui}/${model_name}/${source_name}/${
                neuron_index_n != 0 ? layer_index_n : layer_index_n - 1
              }/${neuron_index_n != 0 ? neuron_index_n - 1 : last_neuron}`,
          (layer_index_n == last_layer) & (neuron_index_n == last_neuron)
            ? alert("This is the last neuron in the model.")
            : `${base_url_ui}${base_ext_ui}/${model_name}/${source_name}/${
                neuron_index_n != last_neuron
                  ? layer_index_n
                  : layer_index_n + 1
              }/${neuron_index_n != last_neuron ? neuron_index_n + 1 : 0}`,
          ,
          `${base_url_ui}${base_ext_ui}/${model_name}/${source_name}/${layer_index_n}`,
          `${base_url_ui}${base_ext_ui}/${model_name}/${source_name}`,
        ];
        surrounding_neurons.innerHTML = `<td class='meta-data' data-tooltip='Visit the current model page'><a href='${model_url}'>Model</a></td><td class='meta-data first' data-tooltip='Visit the previous neuron page'><a href='${prev_url}'>Previous</a></td><td class='meta-data' data-tooltip='Visit the current layer page'><a href='${layer_url}'>Layer</a></td><td class='meta-data' data-tooltip='Visit the next neuron page'><a href='${next_url}'>Next</a></td>`;
        document
          .getElementById("meta-information")
          .appendChild(surrounding_neurons);
      }
    });
}

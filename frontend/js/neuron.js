// When hovering, make tooltip pick that location and show the data-tooltip attribute
// When not hovering, make tooltip disappear
const tooltip = document.getElementById("tooltip");
let target = null;
document.addEventListener("mousemove", (e) => {
  target = e.target;
  if (target.hasAttribute("data-tooltip")) {
    tooltip.style.left = e.pageX + 10 + "px";
    tooltip.style.top = e.pageY + 10 + "px";
    tooltip.style.display = "block";
    tooltip.textContent = target.getAttribute("data-tooltip");
  } else {
    tooltip.style.display = "none";
  }
});


if (serviceName == "all") {
  // Fetch data from the server
  fetch(
    `${baseUrl}/${baseExtApi}/${modelName}/${serviceName}/${layerIndex}/${neuronIndex}`
  )
    .then((response) => { console.log("response", response); return response.json(); })
    .then((data) => {
      // If Neuron2Graph data is available
      if (data["neuron2graph"] != null && data.neuron2graph["data"] != null) {
        neuron2graph_data = data.neuron2graph.data;
        Viz.instance().then(function (viz) {
          let svg = document.body.appendChild(
            viz.renderSVGElement(neuron2graph_data.graph)
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

      if (data["neuron2graph"] != null && data.neuron2graph["data"] != null && data.neuron2graph.data["similar"] != null) {
        const similar = data.neuron2graph.data.similar;
        if (similar.length > 0) {
          for (let i = 0; i < similar.length; i++) {
            const href = `/${baseExtUi}/${modelName}/${serviceName}/${similar[i].layer}/${similar[i].neuron}`
            const similar_neuron = document.createElement("div");
            similar_neuron.classList.add("similar_neurons");
            const similar_neuron_link = document.createElement("a");
            similar_neuron_link.href = href;
            similar_neuron_link.textContent = `${similar[i].layer}:${similar[i].neuron}`;
            similar_neuron.appendChild(similar_neuron_link);
            similar_neuron.innerHTML += `<span data-tooltip='The similarity score to\nthis neuron'>E${similar[i].similarity}</span>,`;
            document.getElementById("similar").appendChild(similar_neuron);
          }
        } else {
          const no_similar = document.createElement("div");
          no_similar.textContent =
            "No similar neurons exist for this neuron.";
          document.getElementById("similar").appendChild(no_similar);
        }
      } else {
        const not_available = document.createElement("div");
        not_available.classList.add("not_available");
        not_available.textContent =
          "Similar neuron data is not available for this neuron.";
        document.getElementById("similar").appendChild(not_available);
      }

      if (data["neuron-explainer"] != null && data["neuron-explainer"]["data"] != null) {
        const neuronExplainerData = data["neuron-explainer"]["data"]
        // If GPT-4 data is available
        const explanation = neuronExplainerData["explanation"]
        const score = neuronExplainerData["score"]

        const explanationElement = document.createElement("div");
        explanationElement.textContent = `Explanation: ${explanation}`;
        const scoreElement = document.createElement("div");
        scoreElement.textContent = `Score: ${score}`;

        const element = document.createElement("div");
        element.classList.add("neuronExplainer");
        element.appendChild(explanationElement);
        element.appendChild(document.createElement("br"));
        element.appendChild(scoreElement);
        document.getElementById("neuronExplainer").appendChild(element);
      } else {
        // Write in a div with class not_available that the data is not available
        const not_available = document.createElement("div");
        not_available.classList.add("not_available");
        not_available.textContent =
          "The GPT-4 data for this neuron is not available.";
        document.getElementById("neuronExplainer").appendChild(not_available);
      }

      // Add a header for the model name
      const header = document.createElement("div");
      header.id = "header";
      header.classList.add("meta");
      document.getElementById("visualization").appendChild(header);
      // Add a header for the serviceName, layerIndex and neuronIndex
      const meta = document.createElement("div");
      meta.id = "meta";
      meta.classList.add("meta");
      document.getElementById("visualization").appendChild(meta);
      const meta_info = document.createElement("tr");
      meta_info.innerHTML =
        "<td class='meta-data first' data-tooltip='The model name'>" +
        modelName +
        "</td><td class='meta-data' data-tooltip='The data service (all includes\n all available services)'>" +
        serviceName +
        "</td><td class='meta-data' data-tooltip='Layer index in the model (from 0)'>" +
        layerIndex +
        "</td><td class='meta-data' data-tooltip='Neuron index in the layer (from 0)'>" +
        neuronIndex +
        "</td>";
      document.getElementById("meta-information").appendChild(meta_info);

      if (data["metadata"] != null && data["metadata"]["data"] != null) {
        let metadata = data.metadata.data;
        const surroundingNeurons = document.createElement("tr");
        const [layerIndex_n, neuronIndex_n, last_layer, last_neuron] = [
          parseInt(layerIndex),
          parseInt(neuronIndex),
          metadata.num_layers - 1,
          metadata.layer_size - 1,
        ];
        const model_url = `${baseUrl}/${baseExtUi}/${modelName}/${serviceName}`;
        const layer_url = `${baseUrl}/${baseExtUi}/${modelName}/${serviceName}/${layerIndex_n}`;
        const prev_url = ((layerIndex_n == 0) && (neuronIndex_n == 0))
          ? alert("This is the first neuron in the model.")
          : `${baseUrl}/${baseExtUi}/${modelName}/${serviceName}/${neuronIndex_n != 0 ? layerIndex_n : layerIndex_n - 1
          }/${neuronIndex_n != 0 ? neuronIndex_n - 1 : last_neuron}`;
        const next_url = (layerIndex_n == last_layer) & (neuronIndex_n == last_neuron)
          ? alert("This is the last neuron in the model.")
          : `${baseUrl}/${baseExtUi}/${modelName}/${serviceName}/${neuronIndex_n != last_neuron
            ? layerIndex_n
            : layerIndex_n + 1
          }/${neuronIndex_n != last_neuron ? neuronIndex_n + 1 : 0}`;
        surroundingNeurons.innerHTML = `<td class='meta-data first' data-tooltip='Visit the current model page'><a href='${model_url}'>Model</a></td><td class='meta-data' data-tooltip='Visit the current layer page'><a href='${layer_url}'>Layer</a></td><td class='meta-data' data-tooltip='Visit the previous neuron page'><a href='${prev_url}'>Previous</a></td><td class='meta-data' data-tooltip='Visit the next neuron page'><a href='${next_url}'>Next</a></td>`;
        document
          .getElementById("meta-information")
          .appendChild(surroundingNeurons);
      }

      if (data["neuroscope"] != null && data["neuroscope"]["data"] != null) {
        const neuroscopeData = data.neuroscope.data;
        const texts = neuroscopeData.texts;
        for (var j = 0; j < texts.length; j++) {
          // Add a header for the current text
          const header = document.createElement("h2");
          header.classList.add("text-title");
          header.innerHTML =
            "Text " +
            j +
            "<span class='meta-info'>" +
            texts[j].min_activation +
            " to " +
            texts[j].max_activation +
            " activation within the range " +
            texts[j].min_range +
            " to " +
            texts[j].max_range +
            ". Data index " +
            texts[j].data_index +
            ". Max activating token located at index " +
            texts[j].max_activating_token_index +
            " of the text of length " +
            texts[j].tokens.length +
            "." +
            "</span>";
          // "Text {i + 1}";
          document.getElementById("visualization").appendChild(header);
          const token_string = document.createElement("div");
          token_string.id = "token_string_" + j;
          token_string.classList.add("token_string");
          document.getElementById("visualization").appendChild(token_string);

          // Get the tokens and activations for the current text
          const tokens = texts[j].tokens;
          const activations = texts[j].activations;
          const abs_activations = activations.map(Math.abs);

          // Get the index of the token with the maximum activation
          const maxActivationIndex = abs_activations.indexOf(
            Math.max(...abs_activations)
          );

          // Scale for coloring the tokens based on activations
          const colorScale = d3
            .scaleLinear()
            .domain([
              Math.min(...abs_activations),
              Math.max(...abs_activations),
            ])
            .range(["#EFEEFF", "#761C6D", "#CC4346", "#F99006", "#F9FC9C"]);

          // Determine the start and end of the slice
          const start = Math.max(0, maxActivationIndex - 50);
          const end = Math.min(tokens.length, maxActivationIndex + 4 + 1); // "+1" because slice end index is exclusive

          // Get the slice of tokens and activations
          const truncated_tokens = tokens.slice(start, end);
          const truncated_activations = activations.slice(start, end);

          // Add each token to the visualization
          truncated_tokens.forEach((token, i) => {
            const activation = truncated_activations[i];
            document
              .getElementById("token_string_" + j)
              .appendChild(
                generate_token_viz(token, activation, colorScale(activation))
              );
          });

          // Create a collapsible button
          var collapsible = document.createElement("button");
          collapsible.textContent = "💬 Show all tokens in sample";
          collapsible.className = "collapsible";
          document.getElementById("visualization").appendChild(collapsible);

          var content = document.createElement("div");
          content.className = "content";

          // Add each token to the full tokens section
          tokens.forEach((token, i) => {
            content.appendChild(
              generate_token_viz(
                token,
                activations[i],
                colorScale(activations[i])
              )
            );
          });

          document.getElementById("visualization").appendChild(content);

          // Add click event to the collapsible button
          collapsible.addEventListener("click", function () {
            this.classList.toggle("active");
            var content = this.nextElementSibling;
            if (content.style.maxHeight) {
              content.style.maxHeight = null;
            } else {
              content.style.maxHeight = content.scrollHeight + "px";
            }
          });
        }
      } else {
        // Write in a div with class not_available that the data is not available
        const not_available = document.createElement("div");
        not_available.classList.add("not_available");
        not_available.textContent =
          "The max activation dataset examples for this neuron are not available.";
        document.getElementById("neuroscope").appendChild(not_available);
      }
    });
} else {
  // Put an h1 in the #meta that says that only /all/ are supported for visualization
  const supporting = document.createElement("h1");
  console.log("MEME MACHINE");
  supporting.innerHTML =
    serviceName +
    " is not supported. Go to <a href='" +
    baseUrl_ui +
    baseExtUi +
    "" +
    modelName +
    "/all/" +
    layerIndex +
    "/" +
    neuronIndex +
    "'>/all/</a> to visualize everything.";
  document.getElementById("meta").appendChild(supporting);
}

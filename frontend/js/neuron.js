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

function renderMeta(document, data, modelName, serviceName, layerIndex, neuronIndex) {
  // Add a header for the model name
  const header = document.createElement("div");
  header.id = "header";
  header.classList.add("meta");
  document.getElementById("visualization").appendChild(header);
  // Add a header for the serviceName, layerIndex and neuronIndex
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

  const surroundingNeurons = document.createElement("tr");
  const [lastLayer, lastNeuron] = [
    data.num_layers - 1,
    data.layer_size - 1,
  ];
  const modelUrl = `${baseUrl}/${baseExtUi}/${modelName}/${serviceName}`;
  const layerUrl = `${modelUrl}/${layerIndex}`;
  const prevUrl = ((layerIndex == 0) && (neuronIndex == 0))
    ? `${modelUrl}/${lastLayer}/${lastNeuron}`
    : `${modelUrl}/${neuronIndex != 0 ? layerIndex : layerIndex - 1
    }/${neuronIndex != 0 ? neuronIndex - 1 : lastNeuron}`;
  const nextUrl = (layerIndex == lastLayer) & (neuronIndex == lastNeuron)
    ? `${modelUrl}/0/0`
    : `${modelUrl}/${neuronIndex != lastNeuron
      ? layerIndex
      : layerIndex + 1
    }/${neuronIndex != lastNeuron ? neuronIndex + 1 : 0}`;
  surroundingNeurons.innerHTML = `
    <td class='meta-data first' data-tooltip='Visit the current model page'><a href='${modelUrl}'>Model</a></td>
    <td class='meta-data' data-tooltip='Visit the current layer page'><a href='${layerUrl}'>Layer</a></td>
    <td class='meta-data' data-tooltip='Visit the previous neuron page'><a href='${prevUrl}'>Previous</a></td>
    <td class='meta-data' data-tooltip='Visit the next neuron page'><a href='${nextUrl}'>Next</a></td>
  `;
  document
    .getElementById("meta-information")
    .appendChild(surroundingNeurons);
}

function notAvailable(document, elementId, message) {
  const element = document.createElement("div");
  element.classList.add("not_available");
  element.textContent =
    `${message} is not available for this neuron.`;
  document.getElementById(elementId).appendChild(element);
}

function renderNeuron2Graph(document, data) {
  Viz.instance().then(function (viz) {
    let svg = document.body.appendChild(
      viz.renderSVGElement(data.graph)
    );
    document.getElementById("n2g").appendChild(svg);
  });

  const similar = data.similar;
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
    const noSimilar = document.createElement("div");
    noSimilar.textContent =
      "No similar neurons exist for this neuron.";
    document.getElementById("similar").appendChild(noSimilar);
  }
}

function renderNeuronExplainer(document, data) {
  const explanation = data["explanation"]
  const score = data["score"]

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
}

function renderNeuroscope(document, data) {
  const texts = data.texts;
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
          generateTokenViz(token, activation, colorScale(activation))
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
        generateTokenViz(
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
}

if (serviceName == "neuron2graph") {
  fetch(`${baseUrl}/${baseExtApi}/${modelName}/${serviceName}/${layerIndex}/${neuronIndex}`).then(async (response) => {
    if (response.ok) {
      const data = await response.json();
      renderMeta(document, data.metadata, modelName, serviceName, layerIndex, neuronIndex);
      if (data.data != null)
        renderNeuron2Graph(document, data.data);
      else if (data["missing_data_objects"] != null)
        notAvailable(document, "n2g", "Neuron2Graph");
      else
        console.error("Unknown error: ", data);
    } else {
      const error = await response.text();
      const errorElement = document.createElement("div");
      errorElement.classList.add("not_available");
      errorElement.textContent = "Error: " + error;
      document.getElementById("n2g").appendChild(errorElement);
    }
  }
  );
} else if (serviceName == "all") {
  // Fetch data from the server
  fetch(
    `${baseUrl}/${baseExtApi}/${modelName}/${serviceName}/${layerIndex}/${neuronIndex}`
  )
    .then((response) => response.json())
    .then((data) => {

      renderMeta(document, data.metadata.data, modelName, serviceName, layerIndex, neuronIndex);

      if (data["neuron2graph"] != null && data.neuron2graph["data"] != null) {
        renderNeuron2Graph(document, data.neuron2graph.data);
      } else {
        notAvailable(document, "n2g", "Neuron2Graph");
      }

      if (data["neuron-explainer"] != null && data["neuron-explainer"]["data"] != null) {
        renderNeuronExplainer(document, data["neuron-explainer"]["data"])
      } else {
        notAvailable(document, "neuronExplainer", "Neuron explainer data");
      }

      if (data["neuroscope"] != null && data["neuroscope"]["data"] != null) {
        renderNeuroscope(document, data.neuroscope.data);
      } else {
        notAvailable(document, "neuroscope", "Neuroscope data");
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

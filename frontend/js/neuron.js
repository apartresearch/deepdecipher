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

const capitalizeWords = (str) => {
  return str
    .toLowerCase()
    .split(" ")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
};

// Parse model_name, source_name, layer_index, and neuron_index from the URL
const [_, model_name, source_name, layer_index, neuron_index] =
  location.pathname.split("/");

const generate_token_viz = (token, activation, color) => {
  const div = document.createElement("span");
  div.className = "token";
  div.style.backgroundColor = color;
  div.textContent = token;
  div.setAttribute("data-tooltip", token + "\n" + activation);
  return div;
};

// Fetch data from the server
fetch(
  `http://localhost:3000/api/${model_name}/${source_name}/${layer_index}/${neuron_index}`
)
  .then((response) => response.json())
  .then((data) => {
    console.log(data);
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
    const meta_info = document.createElement("h2");
    meta_info.textContent =
      "Model: " +
      model_name +
      "\nSource: " +
      capitalizeWords(source_name) +
      "\nNeuron: " +
      neuron_index +
      " in layer " +
      layer_index;
    document.getElementById("meta").appendChild(meta_info);
    // Add a paragraph for explainer text
    const explainer = document.createElement("p");
    explainer.textContent =
      "This visualization shows the most activating tokens for the neuron. The tokens are colored by their activation value. Hover over a token to see its activation value. These text samples are the samples from the dataset that the neuron activates the most to. They are truncated around the most activating token with a window of [-50, 5]. You can click on the '💬 Show all tokens in sample' button to see all tokens of that sample.";
    document.getElementById("meta").appendChild(explainer);

    for (var j = 0; j < data.texts.length; j++) {
      // Add a header for the current text
      const header = document.createElement("h2");
      header.classList.add("text-title");
      header.innerHTML =
        "Text " +
        j +
        "<span class='meta-info'>" +
        data.texts[j].min_act +
        " to " +
        data.texts[j].max_act +
        " activation within the range " +
        data.texts[j].min_range +
        " to " +
        data.texts[j].max_range +
        ". Data index " +
        data.texts[j].data_index +
        ". Max activating token located at index " +
        data.texts[j].max_activating_token_index +
        " of the text of length " +
        data.texts[j].tokens.length +
        "." +
        "</span>";
      // "Text {i + 1}";
      document.getElementById("visualization").appendChild(header);
      const token_string = document.createElement("div");
      token_string.id = "token_string_" + j;
      token_string.classList.add("token_string");
      document.getElementById("visualization").appendChild(token_string);

      // Get the tokens and activations for the current text
      const tokens = data.texts[j].tokens;
      const activations = data.texts[j].activations;
      const abs_activations = activations.map(Math.abs);

      // Get the index of the token with the maximum activation
      const maxActivationIndex = abs_activations.indexOf(
        Math.max(...abs_activations)
      );

      // Scale for coloring the tokens based on activations
      const colorScale = d3
        .scaleLinear()
        .domain([Math.min(...abs_activations), Math.max(...abs_activations)])
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
          generate_token_viz(token, activations[i], colorScale(activations[i]))
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
  });

const [baseExtUi, baseExtApi] = [
  "viz",
  "api",
];

// Parse model name, service name, layer index, and neuron index from the URL
const [baseUrl, viz, modelName, serviceName, layerIndex, neuronIndex] =
  location.pathname.split("/");

const capitalizeWords = (str) => {
  return str
    .toLowerCase()
    .split(" ")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
};

const generate_token_viz = (token, activation, color) => {
  const div = document.createElement("span");
  div.className = "token";
  div.style.backgroundColor = color;
  div.textContent = token;
  div.setAttribute("data-tooltip", token + "\n" + activation);
  return div;
};

const [baseExtUi, baseExtApi] = [
  "viz",
  "api",
];

// Parse model name, service name, layer index, and neuron index from the URL
const [http, doubleSlash, url, viz, modelName, serviceName, layerIndexString, neuronIndexString] =
  window.location.href.split("/");
const [layerIndex, neuronIndex] = [layerIndexString, neuronIndexString].map((s) => parseInt(s));
const baseUrl = [http, doubleSlash, url].join("/");

const capitalizeWords = (str) => {
  return str
    .toLowerCase()
    .split(" ")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
};

const generateTokenViz = (token, activation, color) => {
  const div = document.createElement("span");
  div.className = "token";
  div.style.backgroundColor = color;
  div.textContent = token;
  div.setAttribute("data-tooltip", token + "\n" + activation);
  return div;
};

const getMetadata = async (modelName, callback) => {
  const url = `${baseUrl}/${baseExtApi}/${modelName}/metadata`;
  fetch(
    url
  ).then((response) => response.json()
  ).then((data) => {
    callback(data.data);
  })
}

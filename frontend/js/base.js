const [base_ext_ui, base_ext_api] = [
  "viz",
  "api",
];

// Parse model_name, service_name, layer_index, and neuron_index from the URL
const [base_url, viz, model_name, service_name, layer_index, neuron_index] =
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

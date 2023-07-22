const [base_url_ui, base_url_api, base_ext_ui, base_ext_api] = [
  "http://localhost:3000",
  "http://localhost:8080",
  "/viz",
  "/api",
];

const capitalizeWords = (str) => {
  return str
    .toLowerCase()
    .split(" ")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
};

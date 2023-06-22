const [base_url, base_ext_ui, base_ext_api] = [
  "http://localhost:3000",
  "/viz/",
  "/api/",
];

const capitalizeWords = (str) => {
  return str
    .toLowerCase()
    .split(" ")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
};

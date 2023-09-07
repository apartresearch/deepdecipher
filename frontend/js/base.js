const [base_url_ui, base_url_api, base_ext_ui, base_ext_api] = [
  "http://localhost:3000",
  "http://localhost:8080",
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

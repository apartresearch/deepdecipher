async function n2gSearch() {
    const searchField = document.getElementById("search");
    const searchTerm = searchField.value;
    const loaderElement = document.getElementById("search-message");

    search(modelName, searchTerm, loaderElement, (results) => {
        const resultsDiv = document.querySelector(".results");
        resultsDiv.innerHTML = "";
        if (results.length == 0) {
            resultsDiv.innerHTML = "No results found";
        } else {
            for (result of results) {
                const layerIndex = result.layer;
                const neuronIndex = result.neuron;
                const resultLink = document.createElement("a");
                resultLink.classList.add("result");
                resultLink.href = `${baseUrl}/${baseExtUi}/${modelName}/all/${layerIndex}/${neuronIndex}`;
                resultLink.target = "_blank";
                resultLink.innerHTML = `${layerIndex}:${neuronIndex} ↗`;
                resultsDiv.appendChild(resultLink);
            }
        }
    });
}

const searchElement = document.getElementById("search");
searchElement.addEventListener("keyup", (event) => {
    if (event.key === "Enter") {
        n2gSearch();
    }
});


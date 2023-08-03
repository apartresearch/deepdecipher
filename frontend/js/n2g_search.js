async function search(modelName, searchTerm, loaderElement, callback) {
    showLoader(loaderElement);
    fetch(
        `${baseUrl}/${baseExtApi}/${modelName}/neuron2graph-search?query=any:${searchTerm
            .toString()
            .toLowerCase()}`
    )
        .then((response) => response.json())
        .then((data) => {
            results = data.data;
            return results;
        }).then((results) => {
            hideLoader(loaderElement, results.length);
            return results;
        }).then(callback)
        .catch((error) => console.error("Error:", error));

}

function showLoader(loaderElement) {
    if (loaderElement) {
        document.getElementById("search-message").innerHTML =
            "Loading... (can take up to 30 seconds)";
    }
    console.log("Loading search results...");
}

function hideLoader(loaderElement, numLoaded) {
    if (loaderElement) {
        document.getElementById("search-message").innerHTML = `Found ${numLoaded} results`;
    }
    console.log(`Finished loading ${numLoaded} search results.`);
}
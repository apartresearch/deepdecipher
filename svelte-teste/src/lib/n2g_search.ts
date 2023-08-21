import { BASE_URL, API_EXT } from "./base";

export async function search(modelName: string, searchTerm: string, loaderCallback: (message: string) => void | null): Promise<Record<string, any> | string> {
    showLoader(loaderCallback);
    const url = `${BASE_URL}/${API_EXT}/${modelName}/neuron2graph-search?query=any:${searchTerm
        .toString()
        .toLowerCase()}`;
    let response = await fetch(
        url
    );
    if (response.ok) {
        const results = (await response.json()).data
        hideLoader(loaderCallback, results.length);
        return results;
    } else {
        return await response.text();
    }
}

function showLoader(loaderCallback: (message: string) => void | null) {
    if (loaderCallback) {
        loaderCallback("Loading... (can take up to 30 seconds)");
    }
    console.log("Loading search results...");
}

function hideLoader(loaderCallback: (message: string) => void | null, numLoaded: number) {
    if (loaderCallback) {
        loaderCallback(`Found ${numLoaded} results`);
    }
    console.log(`Finished loading ${numLoaded} search results.`);
}
import { error } from '@sveltejs/kit';
import { BASE_API_URL, API_EXT, VIZ_EXT } from '$lib/base';
import { modelMetadataFromJson } from '$lib/modelMetadata';
import type { Data } from './data';

export async function load({ params }: { params: { model: string, service: string, layer: string, neuron: string } }) {
    // Check indices.
    const layerIndex: number = parseInt(params.layer);
    if (layerIndex < 0 || isNaN(layerIndex)) {
        throw error(404, "Layer index must be a non-negative integer");
    }
    const neuronIndex: number = parseInt(params.neuron);
    if (neuronIndex < 0 || isNaN(layerIndex)) {
        throw error(404, "Neuron index must be a non-negative integer");
    }
    // Get model metadata.
    const url = `${BASE_API_URL}/${API_EXT}/${params.model}/metadata/${layerIndex}/${neuronIndex}`;
    const response = await fetch(
        url
    );
    if (!response.ok) {
        throw error(500, await response.text());
    }
    const json = await response.json();
    let metadata = modelMetadataFromJson(json.data);

    // Check indices against metadata.
    if (typeof metadata == 'string')
        throw error(500, `Model metadata couldn't be loaded. Error: ${metadata}`);
    if (layerIndex >= metadata.numLayers || layerIndex < 0)
        throw error(
            404,
            `Layer index ${layerIndex} is out of bounds. Model has ${metadata.numLayers} layers.`
        );
    if (neuronIndex >= metadata.layerSize || neuronIndex < 0)
        throw error(
            404,
            `Neuron index ${neuronIndex} is out of bounds. Layer has ${metadata.layerSize} neurons.`
        );

    // Create useful URLs.
    const modelUrl = `/${VIZ_EXT}/${params.model}/${params.service}`;
    const layerUrl = `${modelUrl}/${layerIndex}`;
    let prevUrl = '';
    if (neuronIndex > 0) {
        prevUrl = `${layerUrl}/${neuronIndex - 1}`;
    } else if (layerIndex > 0) {
        prevUrl = `${modelUrl}/${layerIndex - 1}/${metadata.layerSize - 1}`;
    } else {
        prevUrl = `${modelUrl}/${metadata.numLayers - 1}/${metadata.layerSize - 1}`;
    }

    let nextUrl = '';
    if (neuronIndex < metadata.layerSize - 1) {
        nextUrl = `${layerUrl}/${neuronIndex + 1}`;
    } else if (layerIndex < metadata.numLayers - 1) {
        nextUrl = `${modelUrl}/${layerIndex + 1}/0`;
    } else {
        nextUrl = `${modelUrl}/0/0`;
    }
    const data: Data = { modelName: params.model, serviceName: params.service, layerIndex, neuronIndex, modelMetadata: metadata, modelUrl, layerUrl, prevUrl, nextUrl };
    return data;
}

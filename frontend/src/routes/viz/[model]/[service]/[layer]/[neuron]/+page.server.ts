import { error } from '@sveltejs/kit';
import { BASE_API_URL, API_EXT, VIZ_EXT } from '$lib/base';
import { modelMetadataFromJson } from '$lib/modelMetadata';
import type { Data } from './data';

export async function load({ params }: { params: { model: string, service: string, layer: string, neuron: string } }) {
    const url = `${BASE_API_URL}/${API_EXT}/${params.model}/all/${params.layer}/${params.neuron}`;
    const response = await fetch(
        url
    );
    if (!response.ok) {
        throw error(500, await response.text());
    }
    const json = await response.json();
    let metadata = modelMetadataFromJson(json.metadata.data);
    let layerIndex = parseInt(params.layer);
    let neuronIndex = parseInt(params.neuron);

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
    const data: Data = { modelName: params.model, serviceName: params.service, layerIndex, neuronIndex, modelMetadata: metadata, services: json, modelUrl, layerUrl, prevUrl, nextUrl };
    return data;
}

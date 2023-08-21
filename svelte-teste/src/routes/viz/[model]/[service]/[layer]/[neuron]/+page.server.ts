import { error } from '@sveltejs/kit';
import { BASE_API_URL, BASE_EXT_API } from '$lib/base';
import { getModelMetadata, modelMetadataFromJson } from '$lib/modelMetadata';
import type { Data } from './data';

export async function load({ params }: { params: { model: string, service: string, layer: number, neuron: number } }) {
    const url = `${BASE_API_URL}/${BASE_EXT_API}/${params.model}/all/${params.layer}/${params.neuron}`;
    const response = await fetch(
        url
    );
    if (!response.ok) {
        return error(500, await response.text());
    }
    const json = await response.json();
    let metadata = modelMetadataFromJson(json.metadata);
    const data: Data = { modelName: params.model, serviceName: params.service, layerIndex: params.layer, neuronIndex: params.neuron, modelMetadata: metadata, services: json };
    return data;
}
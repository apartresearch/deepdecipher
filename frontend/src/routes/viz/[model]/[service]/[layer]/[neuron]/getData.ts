
import { BASE_API_URL, API_EXT, VIZ_EXT } from '$lib/base';
import { error } from '@sveltejs/kit';

export async function getServiceData(modelName: String, service: String, layerIndex: number, neuronIndex: number) {
    const url = `${BASE_API_URL}/${API_EXT}/${modelName}/${service}/${layerIndex}/${neuronIndex}`;
    const response = await fetch(
        url
    );
    if (!response.ok) {
        throw error(500, await response.text());
    }
    return await response.json();
}
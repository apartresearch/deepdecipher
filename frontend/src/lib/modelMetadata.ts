import { BASE_API_URL, API_EXT } from "./base";

export type ModelMetadata = {
    name: string;
    activationFunction: string;
    dataset: string;
    numLayers: number;
    layerSize: number;
    numTotalNeurons: number;
    numTotlalParameters: number;
    availableServices: string[];
}

export function modelMetadataFromJson(json: any): ModelMetadata {
    return {
        name: json["name"],
        activationFunction: json["activation_function"],
        dataset: json["dataset"],
        numLayers: json["num_layers"],
        layerSize: json["layer_size"],
        numTotalNeurons: json["num_total_neurons"],
        numTotlalParameters: json["num_total_parameters"],
        availableServices: json["available_services"]
    }
}

export async function getModelMetadata(modelName: string): Promise<ModelMetadata | string> {
    const url = `${BASE_API_URL}/${API_EXT}/${modelName}/metadata`;
    const response = await fetch(
        url
    );
    if (response.ok) {
        return modelMetadataFromJson((await response.json()).data);
    } else {
        return response.text();
    }
}

export async function getModels(): Promise<ModelMetadata[] | string> {
    const url = `${BASE_API_URL}/${API_EXT}`;
    const response = await fetch(
        url
    );
    if (response.ok) {
        const modelsJson: Record<string, any>[] = (await response.json()).models;
        return modelsJson.map((model: Record<string, any>) => modelMetadataFromJson(model));
    } else {
        return response.text();
    }
}

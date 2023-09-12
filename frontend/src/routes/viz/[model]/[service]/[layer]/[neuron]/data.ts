import type { ModelMetadata } from "$lib/modelMetadata";

export type Data = {
    modelName: string;
    serviceName: string;
    layerIndex: number;
    neuronIndex: number;
    modelMetadata: ModelMetadata;
    modelUrl: string;
    layerUrl: string;
    prevUrl: string;
    nextUrl: string;
}

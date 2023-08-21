import type { ModelMetadata } from "$lib/modelMetadata";

export type Data = {
    modelName: string;
    serviceName: string;
    layerIndex: number;
    neuronIndex: number;
    services: Record<string, any>;
    modelMetadata: ModelMetadata | string;
    modelUrl: string;
    layerUrl: string;
    prevUrl: string;
    nextUrl: string;
}

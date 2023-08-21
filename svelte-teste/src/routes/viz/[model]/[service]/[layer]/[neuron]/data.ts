import type { ModelMetadata } from "../../../../../../modelMetadata";

export type Data = {
    modelName: string;
    serviceName: string;
    layerIndex: number;
    neuronIndex: number;
    services: Record<string, any>;
    modelMetadata: ModelMetadata | string;
}

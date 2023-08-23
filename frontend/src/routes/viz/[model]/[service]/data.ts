import type { ModelMetadata } from "$lib/modelMetadata";

export type Data = {
    modelName: string;
    serviceName: string;
    modelMetadata: ModelMetadata | string;
}

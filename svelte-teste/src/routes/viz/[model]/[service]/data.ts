import type { ModelMetadata } from "../../../../modelMetadata";

export type Data = {
    modelName: string;
    serviceName: string;
    modelMetadata: ModelMetadata | string;
}

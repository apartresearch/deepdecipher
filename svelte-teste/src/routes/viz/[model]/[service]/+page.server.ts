import { getModelMetadata } from '$lib/modelMetadata';
import type { Data } from './data';

export async function load({ params }: { params: { model: string, service: string } }) {
    let metadata = await getModelMetadata(params.model);
    const data: Data = { modelName: params.model, serviceName: params.service, modelMetadata: metadata };
    return data;
}

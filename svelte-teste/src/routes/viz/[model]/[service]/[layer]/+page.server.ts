export function load({ params }: { params: { model: string, service: string, layer: number } }) {
    return { modelName: params.model, serviceName: params.service, layerIndex: params.layer };
}
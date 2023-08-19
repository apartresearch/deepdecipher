export function load({ params }: { params: { model: string, service: string, layer: number, neuron: number } }) {
    return { modelName: params.model, serviceName: params.service, layerIndex: params.layer, neuronIndex: params.neuron };
}
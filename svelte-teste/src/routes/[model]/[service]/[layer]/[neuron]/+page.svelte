<script lang="ts">
	import { error } from '@sveltejs/kit';
	import { BASE_API_URL, BASE_EXT_API, BASE_VIZ_API } from '../../../../../base';
	import Neuron2Graph from './Neuron2Graph.svelte';
	import type { Data } from './data';
	import SimilarNeurons from './SimilarNeurons.svelte';
	import Gpt4Explanation from './Gpt4Explanation.svelte';
	import NotAvailable from './NotAvailable.svelte';

	export let data: Data;

	let { modelName, serviceName, layerIndex, neuronIndex, modelMetadata, services } = data;
	if (typeof modelMetadata == 'string')
		throw error(500, `Model metadata couldn't be loaded. Error: ${modelMetadata}`);
	if (layerIndex >= modelMetadata.numLayers || layerIndex < 0)
		throw error(
			404,
			`Layer index ${layerIndex} is out of bounds. Model has ${modelMetadata.numLayers} layers.`
		);
	if (neuronIndex >= modelMetadata.layerSize || neuronIndex < 0)
		throw error(
			404,
			`Neuron index ${neuronIndex} is out of bounds. Layer has ${modelMetadata.layerSize} neurons.`
		);
	const modelUrl = `/${BASE_VIZ_API}/${modelName}/${serviceName}`;
	const layerUrl = `${modelUrl}/${layerIndex}`;
	let prevUrl = '';
	if (neuronIndex > 0) {
		prevUrl = `${layerUrl}/${neuronIndex - 1}`;
	} else if (layerIndex > 0) {
		prevUrl = `${modelUrl}/${layerIndex - 1}/${modelMetadata.layerSize - 1}`;
	} else {
		prevUrl = `${modelUrl}/${modelMetadata.numLayers - 1}/${modelMetadata.layerSize - 1}`;
	}

	let nextUrl = '';
	if (neuronIndex < modelMetadata.layerSize - 1) {
		nextUrl = `${layerUrl}/${neuronIndex + 1}`;
	} else if (layerIndex < modelMetadata.numLayers - 1) {
		nextUrl = `${modelUrl}/${layerIndex + 1}/0`;
	} else {
		nextUrl = `${modelUrl}/0/0`;
	}

	const neuron2graphData = services['neuron2graph'];
	const neuron2graph = services['neuron2graph'].data.graph;
	const similarNeurons = services['neuron2graph'].data.similar;
	const gpt4Data = services['neuron-explainer'];
</script>

<div class="container">
	<div id="meta">
		<h1>DeepDecipher</h1>
		<table id="meta-information">
			<tr>
				<td class="meta-data first" data-tooltip="The model name">{modelName}</td>
				<td class="meta-data" data-tooltip="The service (all includes all available services)"
					>{serviceName}</td
				>
				<td class="meta-data" data-tooltip="The layer index">{layerIndex}</td>
				<td class="meta-data" data-tooltip="The neuron index">{neuronIndex}</td>
			</tr>
			<tr>
				<td class="meta-data first" data-tooltip="Visit the current model page"
					><a href={modelUrl}>Model</a></td
				>
				<td class="meta-data" data-tooltip="Visit the current layer page"
					><a href={layerUrl}>Layer</a></td
				>
				<td class="meta-data" data-tooltip="Visit the previous neuron page"
					><a href={prevUrl}>Previous</a></td
				>
				<td class="meta-data" data-tooltip="Visit the next neuron page"
					><a href={nextUrl}>Next</a></td
				>
			</tr>
		</table>
	</div>
	<div>
		<h2 class="section-header">Similar neurons</h2>
		{#if 'data' in neuron2graphData}
			<SimilarNeurons similarNeurons={neuron2graphData.data.similar} {modelName} {serviceName} />
		{:else}
			<NotAvailable message="Similar neurons" />
		{/if}
	</div>
	<div id="n2g">
		<h2 class="section-header">
			Neuron semantic graph
			<a href="https://n2g.apartresearch.com">Read what this is</a>
		</h2>
		{#if 'data' in neuron2graphData}
			<Neuron2Graph graphString={neuron2graphData.data.graph} />
		{:else}
			<NotAvailable message="Neuron semantic graph" />
		{/if}
	</div>
	<div id="neuronExplainer">
		<h2 class="section-header">
			Neuron explanation by GPT-4
			<a href="https://openaipublic.blob.core.windows.net/neuron-explainer/paper/index.html"
				>Read what this is</a
			>
		</h2>
		{#if 'data' in gpt4Data}
			<Gpt4Explanation gpt4ExplanationData={gpt4Data.data} />
		{:else}
			<NotAvailable message="Neuron explanation by GPT-4" />
		{/if}
	</div>
	<div id="neuroscope">
		<h2 class="section-header">Max activating dataset examples for this neuron</h2>
		<div id="visualization" />
	</div>
	<div id="tooltip" />
</div>

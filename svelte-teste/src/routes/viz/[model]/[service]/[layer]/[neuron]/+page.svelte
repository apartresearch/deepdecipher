<script lang="ts">
	import { error } from '@sveltejs/kit';
	import { VIZ_EXT } from '$lib/base';
	import Neuron2Graph from './Neuron2Graph.svelte';
	import type { Data } from './data';
	import SimilarNeurons from './SimilarNeurons.svelte';
	import Gpt4Explanation from './Gpt4Explanation.svelte';
	import Neuroscope from './Neuroscope.svelte';

	export let data: Data;

	$: ({
		modelName,
		serviceName,
		layerIndex,
		neuronIndex,
		modelMetadata,
		services,
		modelUrl,
		layerUrl,
		prevUrl,
		nextUrl
	} = data);

	$: neuron2graphData = services['neuron2graph'];
	$: gpt4Data = services['neuron-explainer'];
	$: neuroscope = services['neuroscope'];
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
			<div class="not-available">Similar neurons are not available for this neuron.</div>
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
			<div class="not-available">Neuron semantic graph is not available for this neuron.</div>
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
			<div class="not-available">Neuron explanation by GPT-4 is not available for this neuron.</div>
		{/if}
	</div>
	<div id="neuroscope">
		<h2 class="section-header">Max activating dataset examples for this neuron</h2>

		{#if 'data' in neuroscope}
			<Neuroscope texts={neuroscope.data.texts} />
		{:else}
			<div class="not-available">Neuroscope data is not available for this neuron.</div>
		{/if}
	</div>
	<div id="tooltip" />
</div>

<style>
	.section-header {
		margin: 0.5em 0;
		padding: 0;
		font-size: 1.2em;
		line-height: 1em;
		white-space: wrap;
	}
</style>

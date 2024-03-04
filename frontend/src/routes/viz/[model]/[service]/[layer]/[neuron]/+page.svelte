<script lang="ts">
	import { onMount } from 'svelte';
	import { error } from '@sveltejs/kit';
	import { VIZ_EXT } from '$lib/base';
	import Neuron2Graph from './Neuron2Graph.svelte';
	import type { Data } from './data';
	import { getServiceData } from './getData';
	import SimilarNeurons from './SimilarNeurons.svelte';
	import Gpt4Explanation from './Gpt4Explanation.svelte';
	import Neuroscope from './Neuroscope.svelte';
	import Nav from '$lib/Nav.svelte';
	import NeuronChooser from '../../NeuronChooser.svelte';
	import { goto } from '$app/navigation';

	export let data: Data;

	$: ({
		modelName,
		serviceName,
		layerIndex,
		neuronIndex,
		modelMetadata,
		modelUrl,
		layerUrl,
		prevUrl,
		nextUrl
	} = data);

	$: availableServices = modelMetadata.availableServices;

	let neuron2graphFuture: any = null;

	onMount(() => {
		neuron2graphFuture = getServiceData(modelName, 'neuron2graph', layerIndex, neuronIndex);
	});

	function navPrevNeuron() {
		goto(prevUrl);
	}

	function navNextNeuron() {
		goto(nextUrl);
	}
</script>

<Nav />
<div class="container">
	<h1>
		<a href={modelUrl}>{modelMetadata.name}</a> (L{layerIndex}, N{neuronIndex})
	</h1>
	<div class="action-buttons">
		<button on:click={navPrevNeuron}>Previous</button>
		<button on:click={navNextNeuron}>Next</button>
	</div>
	{#if availableServices.includes('neuron2graph')}
		{#if neuron2graphFuture !== null}
			{#await getServiceData(modelName, 'neuron2graph', layerIndex, neuronIndex)}
				<div>
					<h2 class="section-header">Similar neurons</h2>
					<div>Fetching neuron2graph data...</div>
				</div>
				<div id="n2g">
					<h2 class="section-header">
						Neuron semantic graph
						<a href="https://n2g.apartresearch.com">Read what this is</a>
					</h2>
					<div>Fetching neuron2graph data...</div>
				</div>
			{:then neuron2graphData}
				<div>
					<h2 class="section-header">Similar neurons</h2>

					{#if 'data' in neuron2graphData}
						<SimilarNeurons similarNeurons={neuron2graphData.data.similar} {modelName} />
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
						<Neuron2Graph graphString={neuron2graphData.data.graphviz} />
					{:else}
						<div class="not-available">Neuron semantic graph is not available for this neuron.</div>
					{/if}
				</div>
			{:catch error}
				<div>
					<h2 class="section-header">Similar neurons</h2>
					<div class="not-available">Error occurred while fetching neuron2graph data: {error}</div>
				</div>
				<div id="n2g">
					<h2 class="section-header">
						Neuron semantic graph
						<a href="https://n2g.apartresearch.com">Read what this is</a>
					</h2>
					<div class="not-available">Error occurred while fetching neuron2graph data: {error}</div>
				</div>
			{/await}
		{/if}
	{/if}
	{#if availableServices.includes('neuron_explainer')}
		<div id="neuronExplainer">
			<h2 class="section-header">
				Neuron explanation by GPT-4
				<a href="https://openaipublic.blob.core.windows.net/neuron-explainer/paper/index.html"
					>Read what this is</a
				>
			</h2>
			{#await getServiceData(modelName, 'neuron_explainer', layerIndex, neuronIndex)}
				<div>Fetching data for neuron explanation by GPT-4...</div>
			{:then gpt4Data}
				{#if 'data' in gpt4Data}
					<Gpt4Explanation gpt4ExplanationData={gpt4Data.data} />
				{:else}
					<div class="not-available">
						Neuron explanation by GPT-4 is not available for this neuron.
					</div>
				{/if}
			{:catch error}
				<div class="not-available">
					Error occurred while fetching data for neuron explanation by GPT-4: {error}
				</div>
			{/await}
		</div>
	{/if}
	{#if availableServices.includes('neuroscope')}
		<div id="neuroscope">
			<h2 class="section-header">Max activating dataset examples for this neuron</h2>

			{#await getServiceData(modelName, 'neuroscope', layerIndex, neuronIndex)}
				<div>Fetching neuroscope data...</div>
			{:then neuroscopeData}
				{#if 'data' in neuroscopeData}
					<Neuroscope texts={neuroscopeData.data.texts} />
				{:else}
					<div class="not-available">Neuroscope data is not available for this neuron.</div>
				{/if}
			{:catch error}
				<div class="not-available">Error occurred while fetching neuroscope data: {error}</div>
			{/await}
		</div>
	{/if}
	<div id="tooltip" />
</div>

<style>
	.container {
		max-width: 1140px;
		margin: 0 auto;
		padding: 0 20px;
	}
	h1 {
		margin-bottom: 0;
		font-size: 24px;
	}

	.action-buttons {
		display: flex;
		align-items: center;
		justify-content: flex-start;
		margin-top: 5px;
		padding-bottom: 4px;
		gap: 10px;
	}

	.action-buttons button {
		padding: 8px 16px;
		background-color: #007bff;
		color: white;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		transition: background-color 0.3s ease;
	}

	.action-buttons button:hover {
		background-color: #0056b3;
	}

	.loading-text {
		margin-left: 10px;
		color: #007bff;
	}

	/* Responsive adjustments */
	@media (max-width: 768px) {
		.model-form {
			width: 90%;
		}
	}
</style>

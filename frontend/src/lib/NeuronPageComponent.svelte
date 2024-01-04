<script lang="ts">
	import { onMount } from 'svelte';
	// import { error } from '@sveltejs/kit';
	// import { VIZ_EXT } from '$lib/base';
	// import Neuron2Graph from './Neuron2Graph.svelte';
	// import type { Data } from './data';
	// import { getServiceData } from './getData';
	// import SimilarNeurons from './SimilarNeurons.svelte';
	// import Gpt4Explanation from './Gpt4Explanation.svelte';
	// import Neuroscope from './Neuroscope.svelte';
	// import Title from '$lib/Title.svelte';
	// import NeuronChooser from '../../NeuronChooser.svelte';
	// import { goto } from '$app/navigation';

	// Updating all above imports to fit with the new file structure
	import { onMount } from 'svelte';
	import { error } from '@sveltejs/kit';
	import { VIZ_EXT } from '$lib/base';
	import Neuron2Graph from '$lib/Neuron2Graph.svelte';
	import type { Data } from '$lib/data';
	import { getServiceData } from '$lib/getData';
	import SimilarNeurons from '$lib/SimilarNeurons.svelte';
	import Gpt4Explanation from '$lib/Gpt4Explanation.svelte';
	import Neuroscope from '$lib/Neuroscope.svelte';
	import Title from '$lib/Title.svelte';
	import NeuronChooser from '$lib/NeuronChooser.svelte';
	import { goto } from '$app/navigation';

    // This means that the following needs to be moved to Lib:

	// 	export let data: Data;

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

<div class="container">
	<div id="meta">
		<h1><Title /></h1>
		<table id="metadata-table">
			<tr><th>Model</th><th>Layer</th><th>Neuron</th></tr>
			<tr
				><td><a href={modelUrl}>{modelMetadata.name}</a></td><td>{layerIndex}</td><td
					>{neuronIndex}</td
				><td><button on:click={navPrevNeuron}>Previous</button></td><td
					><button on:click={navNextNeuron}>Next</button></td
				></tr
			>
		</table>
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
						<Neuron2Graph graphString={neuron2graphData.data.graph} />
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
	#metadata-table tr td,
	#metadata-table tr th {
		padding-right: 10px;
	}
	.section-header {
		margin: 0.5em 0;
		padding: 0;
		font-size: 1.2em;
		line-height: 1em;
		white-space: wrap;
	}
</style>
	}
</style>

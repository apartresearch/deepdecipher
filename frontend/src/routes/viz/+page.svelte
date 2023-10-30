<script lang="ts">
	import type { ModelMetadata } from '$lib/modelMetadata';
	import { VIZ_EXT } from '$lib/base';
	import Title from '$lib/Title.svelte';

	export let data: { models: ModelMetadata[] };

	$: models = data.models;
</script>

<h1><Title /></h1>
<p>
	A web page and API that provides interpretability information from many sources on various
	transformer models.
</p>
<table>
	<tr>
		<!-- Removed while article is under review. Should be re-added as soon as possible
			<td style="padding: 0 1em;">
				<a href="https://github.com/apartresearch/deepdecipher">GitHub</a>
			</td>
		-->
		<td style="padding: 0 1em;"><a href="/doc">API documentation</a></td>
	</tr>
</table>
<h2>Features</h2>
<table class="table">
	<thead>
		<tr>
			<th>Feature</th>
			<th>Description</th>
			<th>Source</th>
		</tr>
	</thead>
	<tbody>
		<tr class="model-table-row">
			<td>Neuron2Graph</td>
			<td
				>Vizualize the activation patterns of neurons as a graph. Each path through the graph is a
				n-gram which activates the neuron. From this we also derive a set of similar neurons, which
				are neurons whose graphs are sufficiently similar.
			</td>
			<td><a href="https://n2g.apartresearch.com/">Paper</a></td>
		</tr>
		<tr class="model-table-row">
			<td>Neuroscope</td>
			<td
				>Shows how much the neuron activates to each token in a series of text examples. The
				examples chosen are the examples with the highest activations for that neuron.
			</td><td><a href="neuroscope.io">Website</a></td>
		</tr>
		<tr class="model-table-row">
			<td>Neuron explanation</td>
			<td
				>An attempt by GPT-4 to explain what concept the neuron activates on. Only available for
				models <a href="/viz/gpt2-small"><code>gpt2-small</code></a> and
				<a href="/viz/gpt2-xl"><code>gpt2-xl</code></a>.</td
			>
			<td
				><a
					href="https://openai.com/research/language-models-can-explain-neurons-in-language-models"
					>Website</a
				></td
			>
		</tr>
	</tbody>
</table>
<h2>Available models</h2>
<table class="table">
	<thead>
		<tr class="model-table-row">
			<th>Model</th>
			<th>Activation Function</th>
			<th>Dataset</th>
			<th>Layers</th>
			<th>Neurons per Layer</th>
			<th>Total Neurons</th>
			<th>Total Parameters</th>
			<th>Available Services</th>
		</tr>
	</thead>

	<tbody>
		{#each models as model}
			<tr class="model-table-row">
				<td><a href="/{VIZ_EXT}/{model.name}/all">{model.name}</a></td>
				<td>{model.activationFunction}</td>
				<td>{model.dataset}</td>
				<td>{model.numLayers}</td>
				<td>{model.layerSize.toLocaleString('en-US')}</td>
				<td>{model.numTotalNeurons.toLocaleString('en-US')}</td>
				<td>{model.numTotalParameters.toLocaleString('en-US')}</td>
				<td>{model.availableServices.filter((service) => service !== 'metadata')}</td>
			</tr>
		{/each}
	</tbody>
</table>
<div id="tooltip" />

<style>
	/* General Table Styles */
	.table {
		width: 100%;
		border-collapse: collapse;
		margin-top: 20px;
	}

	/* Header Styles */
	.table thead {
		background-color: #f4f4f4;
	}

	.table th {
		padding: 12px 20px;
		text-align: left;
		font-weight: 600;
	}

	/* Row and Cell Styles */
	.model-table-row {
		border-bottom: 1px solid #ccc;
	}

	.model-table-row td {
		padding: 12px 20px;
		text-align: left;
	}

	/* Hover effect */
	.model-table-row:hover {
		background-color: #f5f5f5;
	}

	/* Tooltip */
	#tooltip {
		position: absolute;
		background-color: #333;
		color: #fff;
		padding: 5px;
		border-radius: 3px;
		font-size: 12px;
	}

	/* Links */
	a {
		color: #0062ff;
		text-decoration: none;
	}

	a:hover {
		text-decoration: underline;
	}
</style>

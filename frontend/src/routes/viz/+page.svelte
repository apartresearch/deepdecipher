<script lang="ts">
	import type { ModelMetadata } from '$lib/modelMetadata';
	import { VIZ_EXT } from '$lib/base';
	import Title from '$lib/Title.svelte';

	export let data: { models: ModelMetadata[] };

	$: models = data.models;
</script>

<h1><Title /> front page</h1>
<p>
	A web page and API that provides interpretability information from many sources on various
	transformer models.
</p>
<table>
	<tr>
		<td style="padding: 0 1em;"
			><a href="https://github.com/apartresearch/deepdecipher">GitHub</a></td
		>
		<td style="padding: 0 1em;"><a href="/doc">API documentation</a></td>
	</tr>
</table>
<table id="model-table">
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
				<td>{model.numTotlalParameters.toLocaleString('en-US')}</td>
				<td>{model.availableServices.filter((service) => service !== 'metadata')}</td>
			</tr>
		{/each}
	</tbody>
</table>
<div id="tooltip" />

<style>
	#model-table,
	.model-table-row th,
	.model-table-row td {
		border: 1px solid;
	}
</style>

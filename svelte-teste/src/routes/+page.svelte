<script>
	import { error } from '@sveltejs/kit';
	import { getModels } from '$lib/modelMetadata';

	async function models() {
		const models = await getModels();
		if (typeof models == 'string') {
			throw error(500, models);
		}
		return models;
	}
</script>

<h1>DeepDecipher front page</h1>
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
	<tr>
		<th>Model</th>
		<th>Activation Function</th>
		<th>Dataset</th>
		<th>Layers</th>
		<th>Neurons per Layer</th>
		<th>Total Neurons</th>
		<th>Total Parameters</th>
		<th>Available Services</th>
	</tr>

	{#await models()}
		<tr>
			<td colspan="8">Loading...</td>
		</tr>
	{:then models}
		{#each models as model}
			<tr>
				<td><a href="/{model.name}/all">{model.name}</a></td>
				<td>{model.activationFunction}</td>
				<td>{model.dataset}</td>
				<td>{model.numLayers}</td>
				<td>{model.layerSize.toLocaleString('en-US')}</td>
				<td>{model.numTotalNeurons.toLocaleString('en-US')}</td>
				<td>{model.numTotlalParameters.toLocaleString('en-US')}</td>
				<td>{model.availableServices}</td>
			</tr>
		{/each}
	{:catch error}
		<tr>
			<td colspan="8">{error.message}</td>
		</tr>
	{/await}
	<tbody id="model-table" />
</table>
<div id="tooltip" />

<style>
	#model-table,
	#model-table tr th,
	#model-table tr td {
		border: 1px solid;
	}
</style>

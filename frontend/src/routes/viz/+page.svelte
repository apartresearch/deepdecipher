<script lang="ts">
	import type { ModelMetadata } from '$lib/modelMetadata';
	import { VIZ_EXT } from '$lib/base';
	import Nav from '$lib/Nav.svelte';
	import CardDescription from '$lib/CardDescription.svelte';

	export let data: { models: ModelMetadata[] };

	$: models = data.models;
</script>

<Nav />
<div class="container">
	<h2>Models</h2>
	<div class="model-cards-container">
		{#each models as model}
			<div class="model-card">
				<a class="full-width" href="/{VIZ_EXT}/{model.name}/all">
					<h3>{model.name}</h3>
					<p class="sublabel">Model name</p>
				</a>
				<div class="features">
					<div>
						<p>
							{model.availableServices.filter((service) => service !== 'metadata').join(', ')}
						</p>
						<p class="sublabel">Available Services</p>
					</div>
					<div>
						<p>{model.activationFunction}</p>
						<p class="sublabel">Activation Function</p>
					</div>
					<div>
						<p>{model.numLayers}</p>
						<p class="sublabel">Layers</p>
					</div>
					<div>
						<p>{model.layerSize.toLocaleString('en-US')}</p>
						<p class="sublabel">Neurons per Layer</p>
					</div>
					<div>
						<p>{model.numTotalNeurons.toLocaleString('en-US')}</p>
						<p class="sublabel">Total Neurons</p>
					</div>
					<div>
						<p>
							{model.numTotalParameters.toLocaleString('en-US')}
						</p>
						<p class="sublabel">Total Parameters</p>
					</div>
					<div>
						<p>{model.dataset}</p>
						<p class="sublabel">Dataset</p>
					</div>
				</div>
			</div>
		{/each}
	</div>
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
					n-gram which activates the neuron. From this we also derive a set of similar neurons,
					which are neurons whose graphs are sufficiently similar.
				</td>
				<td><a href="https://n2g.apartresearch.com/">Paper</a></td>
			</tr>
			<tr class="model-table-row">
				<td>Neuroscope</td>
				<td
					>Shows how much the neuron activates to each token in a series of text examples. The
					examples chosen are the examples with the highest activations for that neuron.
				</td><td><a href="https://neuroscope.io/">Website</a></td>
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
</div>
<div id="tooltip" />

<style>
	.container {
		max-width: 1140px;
		margin: 0 auto;
		color: #333;
	}

	h2 {
		font-size: 24px;
		color: #222;
		margin-top: 20px;
		margin-bottom: 10px;
		font-weight: normal;
	}

	.table {
		width: 100%;
		border-collapse: collapse;
		margin-bottom: 40px;
	}

	.table th,
	.table td {
		text-align: left;
		padding: 12px 15px;
		border-bottom: 1px solid #eaeaea;
	}

	.table th {
		font-weight: 500;
		color: #666;
		background-color: #f9f9f9;
	}

	.table td {
		color: #555;
	}

	.table a {
		color: #007bff;
		text-decoration: none;
	}

	.table a:hover {
		text-decoration: underline;
	}

	.model-table-row:nth-child(odd) {
		background-color: #f9f9f9;
	}

	/* Tooltip Styling */
	#tooltip {
		position: absolute;
		background-color: #fff;
		border: 1px solid #ddd;
		padding: 10px 15px;
		border-radius: 4px;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
		visibility: hidden;
		z-index: 1000;
	}

	.model-cards-container {
		display: flex;
		flex-wrap: wrap;
		gap: 20px;
		justify-content: left;
	}

	.model-card {
		flex: 1;
		min-width: 280px;
		max-width: 340px;
		background-color: #fff;
		box-shadow: 1px 2px 4px rgba(17, 45, 115, 0.1);
		border-radius: 8px;
		padding: 20px 0;
		display: flex;
		flex-wrap: wrap;
		gap: 5px;
		/* Make flex children all the same height */
		align-items: stretch;
	}

	.model-card > .features {
		display: flex;
		flex-wrap: wrap;
		gap: 5px;
		justify-content: space-between;
		margin-top: 5px;
		border-top: 1px solid #eaeaea;
		padding: 0 20px;
		padding-top: 10px;
	}

	.features > div {
		flex: 1;
		min-width: 30%;
		display: flex;
		flex-direction: column-reverse;

		align-items: flex-start;
		/* Make children be at the top */
		justify-content: flex-end;
	}

	.model-card h3 {
		margin: 0;
		font-size: 18px;
		font-weight: 400;
	}

	.model-card a {
		color: #6121eb;
		text-decoration: none;
	}

	.model-card > .full-width {
		padding: 0 20px;
	}

	.model-card a:hover {
		text-decoration: underline;
	}

	.model-card p {
		margin: 0;
		font-size: 12px;
		line-height: 1.25;
	}

	p.sublabel {
		font-size: 12px;
		color: #666;
		margin: 0;
	}

	a.full-width {
		width: 100%;
	}
</style>

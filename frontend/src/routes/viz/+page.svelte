<script lang="ts">
	import type { ModelMetadata } from '$lib/modelMetadata';
	import { VIZ_EXT } from '$lib/base';
	import Nav from '$lib/Nav.svelte';
	import GraphView from '$lib/GraphView.svelte';
	import Footer from '$lib/Footer.svelte';

	export let data: { models: ModelMetadata[] };

	$: models = data.models;
</script>

<Nav />
<div class="paper-wrapper">
	<a href="https://arxiv.org/abs/2310.01870" class="paper"
		><img src="/deepdecipher-paper.png" alt="Paper front page" class="paper-image" />
	</a>
	<div class="column">
		<h2>DeepDecipher: Accessing and Investigating Neuron Activation in Large Language Models</h2>
		<p class="authors">
			Albert Garde<sup>*1</sup>, Esben Kran<sup>*1</sup>, Fazl Barez<sup>1, 2, 3</sup>
		</p>
		<p class="affiliation">
			<sup>1</sup>Apart Research, <sup>2</sup>Edinburgh Centre for Robotics, <sup>3</sup>Department
			of Engineering Sciences, University of Oxford
		</p>
		<p class="correspondence">
			Correspondence to: <a href="mailto:albertsgarde@gmail.com" class="correspondence"
				>albertsgarde@gmail.com</a
			>
		</p>
	</div>
</div>
<div class="graph-columns">
	<GraphView />
</div>

<div id="tooltip" />

<Footer />

<style>
	p {
		margin: 0;
	}

	.graph-columns {
		display: flex;
		flex-direction: row;
		flex-wrap: wrap;
		margin: 0.5em 0.5em;
	}

	.paper-wrapper {
		display: flex;
		flex-direction: row;
		max-width: 570px;
		flex-wrap: wrap;
		column-gap: 1em;
		padding: 0.5em 0.65em;
		margin: 0.5em auto;
		border-radius: 3px;
		border: 1px solid #ccc;
		background-color: #ddd;
	}

	.column {
		flex: 1;
	}

	.paper-image {
		width: 100%;
		max-height: 150px;
		border: 1px solid #ccc;
	}

	.authors {
		font-size: 0.9em;
	}

	.affiliation {
		font-size: 0.9em;
	}

	.correspondence {
		font-size: 0.9em;
	}

	h2 {
		font-size: 1.25em;
		margin: 0.25em 0;
	}
</style>

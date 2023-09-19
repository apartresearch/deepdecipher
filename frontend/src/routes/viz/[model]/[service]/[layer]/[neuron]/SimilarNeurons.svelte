<script lang="ts">
	import NeuronLink from '$lib/NeuronLink.svelte';
	import { VIZ_EXT } from '$lib/base';

	export let similarNeurons: any[];
	export let modelName: string;

	$: similarNeurons = similarNeurons.sort((a, b) => b.similarity - a.similarity);
</script>

<div id="similar">
	{#if similarNeurons.length > 0}
		<table>
			<thead>
				<tr>
					<th>Similarity</th>
					<th>Neuron</th>
				</tr>
			</thead>
			<tbody>
				{#each similarNeurons as { layer, neuron, similarity }}
					<tr><td>{similarity}</td><td><NeuronLink {modelName} {layer} {neuron} /></td></tr>
				{/each}
			</tbody>
		</table>
	{:else}
		<div>No similar neurons exist for this neuron.</div>
	{/if}
</div>

<style>
	#similar {
		text-align: left;
		display: flex;
		flex-wrap: wrap;
		flex-direction: row;
	}
</style>

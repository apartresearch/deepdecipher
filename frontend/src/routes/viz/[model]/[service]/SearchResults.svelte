<script lang="ts">
	import NeuronLink from '$lib/NeuronLink.svelte';

	export let modelName: string;
	export let searchResults: any;
</script>

<div class="results">
	{#if searchResults.length == 0}
		<div>No results found</div>
	{:else}
		{#each searchResults as { layer, neuron }}
			<NeuronLink {modelName} {layer} {neuron} />
		{/each}
	{/if}
</div>

<style>
	.results {
		display: flex;
		flex-wrap: wrap;
		flex-direction: row;
		max-width: 40em;
	}
</style>
